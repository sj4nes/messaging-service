use crate::store_db::inbound_events::{
    claim_batch, fetch_event, mark_error, mark_processed, reap_stale,
};
use crate::store_db::messages::insert_from_inbound;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, instrument, warn};

use crate::config::ApiConfig;
use crate::metrics;

pub struct InboundWorker {
    pool: PgPool,
    cfg: ApiConfig,
}

impl InboundWorker {
    pub fn new(pool: PgPool, cfg: ApiConfig) -> Self {
        Self { pool, cfg }
    }

    #[instrument(skip(self))]
    pub async fn run(self) {
        let batch_size = self.cfg.worker_batch_size as i64;
        loop {
            match claim_batch(&self.pool, batch_size).await {
                Ok(ids) if ids.is_empty() => {
                    sleep(Duration::from_millis(500)).await;
                }
                Ok(ids) => {
                    metrics::record_worker_claimed(ids.len() as u64);
                    for id in ids {
                        if let Err(e) = self.process_one(id).await {
                            // Schedule retry / dead-letter
                            let dead = match mark_error(
                                &self.pool,
                                id,
                                "process_error",
                                &format!("{:?}", e),
                                self.cfg.worker_max_retries as i32,
                                self.cfg.worker_backoff_base_ms as i64,
                            )
                            .await
                            {
                                Ok(is_dead) => is_dead,
                                Err(err) => {
                                    warn!(error=?err, inbound_event_id=id, "failed to mark_error; keeping pending");
                                    false
                                }
                            };
                            if dead {
                                metrics::record_worker_dead_letter();
                            } else {
                                metrics::record_worker_error();
                            }
                            warn!(error=?e, inbound_event_id=id, dead, "worker process_one failed");
                        }
                    }
                }
                Err(e) => {
                    error!(error=?e, "worker claim_batch error");
                    sleep(Duration::from_millis(1000)).await;
                }
            }
            // periodic reap stale
            if let Err(e) = reap_stale(&self.pool, self.cfg.worker_claim_timeout_secs as i64).await
            {
                warn!(error=?e, "worker reap_stale error");
            }
        }
    }

    async fn process_one(&self, inbound_id: i64) -> anyhow::Result<()> {
        let started = std::time::Instant::now();
        if let Some((channel, from, to, payload)) = fetch_event(&self.pool, inbound_id).await? {
            // Minimal parse: attempt body + timestamp fields if present
            let body = payload.get("body").and_then(|v| v.as_str()).unwrap_or("");
            let attachments: Vec<String> = payload
                .get("attachments")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let ts = payload
                .get("timestamp")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
            // Insert placeholder message representation (no actual DB writes yet)
            let _ = insert_from_inbound(
                &self.pool,
                &channel,
                from.as_deref().unwrap_or("unknown"),
                to.as_deref().unwrap_or("unknown"),
                body,
                &attachments,
                &ts,
            )
            .await?;
        }
        mark_processed(&self.pool, inbound_id).await?;
        metrics::record_worker_processed(started.elapsed().as_micros() as u64);
        Ok(())
    }
}
