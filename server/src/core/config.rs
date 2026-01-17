use anyhow::{Context, Result};
use std::env;

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
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_host: String,
    pub db_port: u16,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub db_url: String,
}


#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub port: u16,
    pub host: String,
    pub username: String,
    pub password: String,
    pub key_prefix: String,
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
        let db_user = env::var("DATABASE_USER").context("DATABASE_USER must be set")?;
        let db_password = env::var("DATABASE_PASSWORD").context("DATABASE_PASSWORD must be set")?;
        let db_host = env::var("DATABASE_HOST").context("DATABASE_HOST must be set")?;
        let db_port = env::var("DATABASE_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .context("Invalid DATABASE_PORT value")?;
        let db_name = env::var("DATABASE_NAME").context("DATABASE_NAME must be set")?;
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user.clone(),
            db_password.clone(),
            db_host.clone(),
            db_port,
            db_name.clone(),
        );
        Ok(DatabaseConfig {
            db_url,
            db_name,
            db_user,
            db_password,
            db_host,
            db_port,
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
        let port = env::var("REDIS_PORT")
            .map_err(|_| anyhow::anyhow!("REDIS_PORT must be set"))?
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid REDIS_PORT value"))?;
        let host = env::var("REDIS_HOST")
            .map_err(|_| anyhow::anyhow!("REDIS_HOST must be set"))?;
        let username = env::var("REDIS_USERNAME")
            .map_err(|_| anyhow::anyhow!("REDIS_USERNAME must be set"))?;
        let password = env::var("REDIS_PASSWORD")
            .map_err(|_| anyhow::anyhow!("REDIS_PASSWORD must be set"))?;
        let key_prefix = env::var("REDIS_KEY_PREFIX")
            .unwrap_or_else(|_| "hub_".to_string());
        let url = format!("redis://{}:{}@{}:{}", username, password, host, port );
        Ok(RedisConfig {
            port,
            host,
            username,
            password,
            key_prefix,
            url,
        })
    }
}

impl JwtConfig {
    fn from_env() -> Result<Self> {
        Ok(JwtConfig {
            secret: env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
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
            app_id: env::var("WECHAT_APP_ID").context("WECHAT_APP_ID must be set")?,
            app_secret: env::var("WECHAT_APP_SECRET").context("WECHAT_APP_SECRET must be set")?,
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
