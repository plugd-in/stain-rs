//! Implementation of JSON [Config](crate::stains::Config) source.

use std::num::NonZeroU16;

use anyhow::Result;
use async_trait::async_trait;
use stain::stain;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    AppConfig,
    stains::{Config, config},
};

#[derive(Default)]
struct JsonConfig;

#[async_trait]
impl Config for JsonConfig {
    async fn config(&self, config: &mut AppConfig) -> Result<()> {
        let mut json_file = File::open("examples/async_trait_plugins/config.json").await?;

        let mut json = String::new();
        json_file.read_to_string(&mut json).await?;

        let json = serde_json::from_str::<serde_json::Value>(&json)?;

        if let Some(port) = json.get("port").and_then(|port| port.as_u64()) {
            let port = u16::try_from(port)?;
            let port = NonZeroU16::try_from(port)?;

            config.port = port;
        }

        if let Some(log_level) = json.get("log_level").and_then(|log_level| log_level.as_str()) {
            config.log_level = Box::from(log_level);
        }

        Ok(())
    }
}

stain! {
    store: config;
    item: JsonConfig;
    ordering: 0;
}
