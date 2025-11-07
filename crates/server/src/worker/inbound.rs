use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, instrument, warn};

use crate::config::ApiConfig;
use crate::metrics;
use crate::store_db::inbound_events::{claim_batch, mark_error, mark_processed, reap_stale};
use crate::store_db::messages::insert_from_inbound;

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
                            warn!(error=?e, inbound_event_id=id, "worker process_one failed");
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
        // TODO: fetch event details and create message record(s).
        mark_processed(&self.pool, inbound_id).await?;
        // For now, record zero latency; future work will measure per-event processing time.
        metrics::record_worker_processed(0);
        Ok(())
    }
}
