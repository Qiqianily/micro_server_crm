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
}

impl AppConfig {
    // 加载配置
    pub fn load() -> Result<Self> {
        let ret = match (
            File::open("metadata.yml"),
            File::open("/etc/config/metadata.yml"),
            env::var("METADATA_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("Metadata Config file not found"),
        };

        Ok(ret?)
    }
}
