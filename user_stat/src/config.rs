use std::{env, fs::File};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// 服务配置
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    // 服务器相关
    pub server: ServerConfig,
    // 身份认证相关
    pub auth: AuthConfig,
}

/// 身份认证配置 暂未启用
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    // 私钥
    pub pk: String,
}

// 服务配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    // 监听端口
    pub port: u16,
    // 数据连接
    pub db_url: String,
}

impl AppConfig {
    // 加载配置
    pub fn load() -> Result<Self> {
        let ret = match (
            File::open("user_stat.yml"),
            File::open("/etc/config/user_stat.yml"),
            env::var("USER_STAT_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("User Stat Config file not found"),
        };

        Ok(ret?)
    }
}
