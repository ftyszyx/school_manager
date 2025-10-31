use std::env;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
    pub wechat: WechatConfig,
    pub system: SystemConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct WechatConfig {
    pub app_id: String,
    pub app_secret: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expire_days: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub default_user_password: String,
    pub register_allowed: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            jwt: JwtConfig::from_env()?,
            server: ServerConfig::from_env()?,
            wechat: WechatConfig::from_env()?,
            system: SystemConfig::from_env()?,
        })
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self> {
        Ok(DatabaseConfig {
            db_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("Invalid DB_MAX_CONNECTIONS value")?,
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid DB_MIN_CONNECTIONS value")?,
            connect_timeout_secs: env::var("DB_CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .context("Invalid DB_CONNECT_TIMEOUT value")?,
        })
    }
}

impl RedisConfig {
    fn from_env() -> Result<Self> {
        Ok(RedisConfig {
            url: env::var("REDIS_URL")
                .context("REDIS_URL must be set")?,
        })
    }
}

impl JwtConfig {
    fn from_env() -> Result<Self> {
        Ok(JwtConfig {
            secret: env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            expire_days: env::var("JWT_EXPIRE")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .context("Invalid JWT_EXPIRE value")?,
        })
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self> {
        Ok(ServerConfig {
            host: env::var("LISTEN_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("LISTEN_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("Invalid LISTEN_PORT value")?,
        })
    }
}

impl WechatConfig {
    fn from_env() -> Result<Self> {
        Ok(WechatConfig {
            app_id: env::var("WECHAT_APP_ID")
                .context("WECHAT_APP_ID must be set")?,
            app_secret: env::var("WECHAT_APP_SECRET")
                .context("WECHAT_APP_SECRET must be set")?,
        })
    }
}

impl SystemConfig {
    fn from_env() -> Result<Self> {
        Ok(SystemConfig {
            default_user_password: env::var("DEFAULT_USER_PASSWORD")
                .unwrap_or_else(|_| "123456".to_string()),
            register_allowed: env::var("REGISTER_ALLOWED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid REGISTER_ALLOWED value")?,
        })
    }
}