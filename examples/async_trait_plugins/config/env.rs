//! Implementation of environment variable [Config](crate::stains::Config) source.

use std::num::NonZeroU16;

use anyhow::Result;
use async_trait::async_trait;
use stain::stain;

use crate::{
    AppConfig,
    stains::{Config, config},
};

#[derive(Default)]
struct EnvConfig;

#[async_trait]
impl Config for EnvConfig {
    async fn config(&self, config: &mut AppConfig) -> Result<()> {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .and_then(NonZeroU16::new);

        let log_level = std::env::var("LOG_LEVEL").ok();

        if let Some(port) = port {
            config.port = port;
        }

        if let Some(log_level) = log_level {
            config.log_level = log_level.into_boxed_str();
        }

        Ok(())
    }
}

stain! {
    store: config;
    item: EnvConfig;
    // We use ordering to "prioritize" env config by running it later.
    ordering: 1;
}
