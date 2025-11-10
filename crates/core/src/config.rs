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
    pub conversation_snippet_length: usize,
    // --- Security additions (Feature 010) ---
    pub auth_session_expiry_min: u64,
    pub rate_limit_per_ip_per_min: u32,
    pub rate_limit_per_sender_per_min: u32,
    pub argon2_memory_mb: u32,
    pub argon2_time_cost: u32,
    pub argon2_parallelism: u32,
    pub security_headers_enabled: bool,
    pub csp_default_src: String,
    pub ssrf_allowlist: Vec<String>,
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

        // CONVERSATION_SNIPPET_LENGTH default 64 (must be >= 1 and <= 4096)
        let snippet_len_str =
            env::var("CONVERSATION_SNIPPET_LENGTH").unwrap_or_else(|_| "64".to_string());
        let conversation_snippet_length: usize = snippet_len_str.parse().map_err(|_| {
            format!("Invalid CONVERSATION_SNIPPET_LENGTH value: '{snippet_len_str}'")
        })?;
        if !(1..=4096).contains(&conversation_snippet_length) {
            return Err("CONVERSATION_SNIPPET_LENGTH must be in 1..=4096".to_string());
        }

        // Security: AUTH_SESSION_EXPIRY_MIN default 30
        let auth_session_expiry_min = env::var("AUTH_SESSION_EXPIRY_MIN")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);
        if !(1..=1440).contains(&auth_session_expiry_min) {
            return Err("AUTH_SESSION_EXPIRY_MIN must be in 1..=1440".to_string());
        }

        // Rate limits
        let rate_limit_per_ip_per_min = env::var("API_RATE_LIMIT_PER_IP_PER_MIN")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(120);
        let rate_limit_per_sender_per_min = env::var("API_RATE_LIMIT_PER_SENDER_PER_MIN")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(60);
        if rate_limit_per_ip_per_min == 0 || rate_limit_per_sender_per_min == 0 {
            return Err("Rate limits must be > 0".to_string());
        }

        // Argon2 parameters
        let argon2_memory_mb = env::var("ARGON2_MEMORY_MB")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(64);
        let argon2_time_cost = env::var("ARGON2_TIME_COST")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);
        let argon2_parallelism = env::var("ARGON2_PARALLELISM")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        if argon2_memory_mb < 8 || argon2_memory_mb > 1024 {
            return Err("ARGON2_MEMORY_MB must be in 8..=1024".to_string());
        }
        if !(1..=10).contains(&argon2_time_cost) {
            return Err("ARGON2_TIME_COST must be in 1..=10".to_string());
        }
        if !(1..=32).contains(&argon2_parallelism) {
            return Err("ARGON2_PARALLELISM must be in 1..=32".to_string());
        }

        // Security headers enable flag
        let security_headers_enabled = env::var("SECURITY_HEADERS_ENABLED")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(true);

        // CSP default-src directive
        let csp_default_src = env::var("CSP_DEFAULT_SRC").unwrap_or_else(|_| "'self'".to_string());

        // SSRF allowlist: comma-separated hostnames
        let ssrf_allowlist = env::var("SSRF_ALLOWLIST")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        Ok((
            Config {
                port,
                health_path,
                log_level,
                conversation_snippet_length,
                auth_session_expiry_min,
                rate_limit_per_ip_per_min,
                rate_limit_per_sender_per_min,
                argon2_memory_mb,
                argon2_time_cost,
                argon2_parallelism,
                security_headers_enabled,
                csp_default_src,
                ssrf_allowlist,
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
        let saved = save_and_clear(&[
            "PORT",
            "HEALTH_PATH",
            "LOG_LEVEL",
            "CONVERSATION_SNIPPET_LENGTH",
        ]);
        let cfg = Config::load().expect("load defaults");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.health_path, "/healthz");
        assert_eq!(cfg.log_level, "info");
        assert_eq!(cfg.conversation_snippet_length, 64);
        restore(saved);
    }

    #[rstest]
    fn env_precedence_and_validation() {
        let _lock = TEST_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
        let saved = save_and_clear(&[
            "PORT",
            "HEALTH_PATH",
            "LOG_LEVEL",
            "CONVERSATION_SNIPPET_LENGTH",
        ]);
        std::env::set_var("PORT", "9090");
        std::env::set_var("HEALTH_PATH", "status");
        std::env::set_var("LOG_LEVEL", "debug");
        std::env::set_var("CONVERSATION_SNIPPET_LENGTH", "128");
        let cfg = Config::load().expect("load env");
        assert_eq!(cfg.port, 9090);
        assert_eq!(cfg.health_path, "/status");
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.conversation_snippet_length, 128);
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
