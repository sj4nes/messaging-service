use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub port: u16,
    pub health_path: String,
    pub log_level: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        // Load .env if present; environment variables take precedence over .env
        let _ = dotenvy::dotenv();

        // PORT with default 8080
        let port_str = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let port: u16 = port_str
            .parse()
            .map_err(|_| format!("Invalid PORT value: '{port_str}'"))?;
        if port == 0 {
            return Err("PORT must be in 1..=65535".to_string());
        }

        // HEALTH_PATH with default "/healthz"
        let mut health_path = env::var("HEALTH_PATH").unwrap_or_else(|_| "/healthz".to_string());
        if health_path.trim().is_empty() {
            health_path = "/healthz".to_string();
        }
        if !health_path.starts_with('/') {
            health_path.insert(0, '/');
        }

        // LOG_LEVEL default "info"
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            port,
            health_path,
            log_level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn save_and_clear(keys: &[&str]) -> Vec<(String, Option<String>)> {
        let mut saved = Vec::new();
        for &k in keys {
            let prev = std::env::var(k).ok();
            saved.push((k.to_string(), prev));
            std::env::remove_var(k);
        }
        saved
    }

    fn restore(saved: Vec<(String, Option<String>)>) {
        for (k, v) in saved {
            if let Some(val) = v {
                std::env::set_var(&k, val);
            } else {
                std::env::remove_var(&k);
            }
        }
    }

    #[rstest]
    fn defaults_when_unset() {
        let saved = save_and_clear(&["PORT", "HEALTH_PATH", "LOG_LEVEL"]);
        let cfg = Config::load().expect("load defaults");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.health_path, "/healthz");
        assert_eq!(cfg.log_level, "info");
        restore(saved);
    }

    #[rstest]
    fn env_precedence_and_validation() {
        let saved = save_and_clear(&["PORT", "HEALTH_PATH", "LOG_LEVEL"]);
        std::env::set_var("PORT", "9090");
        std::env::set_var("HEALTH_PATH", "status");
        std::env::set_var("LOG_LEVEL", "debug");
        let cfg = Config::load().expect("load env");
        assert_eq!(cfg.port, 9090);
        assert_eq!(cfg.health_path, "/status");
        assert_eq!(cfg.log_level, "debug");
        restore(saved);
    }
}
