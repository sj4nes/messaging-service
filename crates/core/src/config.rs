use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Env,
    Dotenv,
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSources {
    pub port: Source,
    pub health_path: Source,
    pub log_level: Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub port: u16,
    pub health_path: String,
    pub log_level: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        Ok(Self::load_with_sources()?.0)
    }

    pub fn load_with_sources() -> Result<(Self, ConfigSources), String> {
        // Snapshot env before loading .env to detect whether keys come from real env or .env
        let pre_env_port = env::var_os("PORT").is_some();
        let pre_env_health = env::var_os("HEALTH_PATH").is_some();
        let pre_env_log = env::var_os("LOG_LEVEL").is_some();

        // Load .env if present; environment variables take precedence over .env
        let _ = dotenvy::dotenv();

        // After loading .env, check presence
        let post_env_port = env::var_os("PORT").is_some();
        let post_env_health = env::var_os("HEALTH_PATH").is_some();
        let post_env_log = env::var_os("LOG_LEVEL").is_some();

        // Determine sources
        let src_port = if pre_env_port {
            Source::Env
        } else if post_env_port {
            Source::Dotenv
        } else {
            Source::Default
        };
        let src_health = if pre_env_health {
            Source::Env
        } else if post_env_health {
            Source::Dotenv
        } else {
            Source::Default
        };
        let src_log = if pre_env_log {
            Source::Env
        } else if post_env_log {
            Source::Dotenv
        } else {
            Source::Default
        };

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

        Ok((
            Config {
                port,
                health_path,
                log_level,
            },
            ConfigSources {
                port: src_port,
                health_path: src_health,
                log_level: src_log,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::{
        fs,
        sync::{Mutex, OnceLock},
    };

    static TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

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
        let _lock = TEST_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
        let saved = save_and_clear(&["PORT", "HEALTH_PATH", "LOG_LEVEL"]);
        let cfg = Config::load().expect("load defaults");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.health_path, "/healthz");
        assert_eq!(cfg.log_level, "info");
        restore(saved);
    }

    #[rstest]
    fn env_precedence_and_validation() {
        let _lock = TEST_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
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

    fn with_temp_dotenv(contents: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(".env");
        fs::write(&path, contents).expect("write .env");
        dir
    }

    #[rstest]
    fn dotenv_used_when_env_unset() {
        let _lock = TEST_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
        let saved_env = save_and_clear(&["PORT", "HEALTH_PATH", "LOG_LEVEL"]);
        let cwd = std::env::current_dir().expect("cwd");

        let tmp = with_temp_dotenv("PORT=1234\nHEALTH_PATH=status\nLOG_LEVEL=warn\n");
        std::env::set_current_dir(tmp.path()).expect("chdir");

        let (cfg, src) = Config::load_with_sources().expect("load dotenv");
        assert_eq!(cfg.port, 1234);
        assert_eq!(cfg.health_path, "/status");
        assert_eq!(cfg.log_level, "warn");
        assert_eq!(src.port, Source::Dotenv);
        assert_eq!(src.health_path, Source::Dotenv);
        assert_eq!(src.log_level, Source::Dotenv);

        // restore
        std::env::set_current_dir(cwd).ok();
        restore(saved_env);
    }

    #[rstest]
    fn env_overrides_dotenv_when_both_present() {
        let _lock = TEST_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
        let saved_env = save_and_clear(&["PORT", "HEALTH_PATH", "LOG_LEVEL"]);
        let cwd = std::env::current_dir().expect("cwd");

        let tmp = with_temp_dotenv("PORT=1234\nHEALTH_PATH=status\nLOG_LEVEL=warn\n");
        std::env::set_current_dir(tmp.path()).expect("chdir");

        // Set env vars that should override .env
        std::env::set_var("PORT", "5678");
        std::env::set_var("LOG_LEVEL", "debug");

        let (cfg, src) = Config::load_with_sources().expect("load env over dotenv");
        assert_eq!(cfg.port, 5678);
        assert_eq!(cfg.health_path, "/status"); // from .env as not set in env
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(src.port, Source::Env);
        assert_eq!(src.health_path, Source::Dotenv);
        assert_eq!(src.log_level, Source::Env);

        // restore
        std::env::set_current_dir(cwd).ok();
        restore(saved_env);
    }
}
