// #![allow(unused)]

use std::{io::Write, num::NonZeroU16};

use anyhow::Result;
use stain::Store;

mod config;
mod stains;

#[derive(Debug)]
struct AppConfig {
    port: NonZeroU16,
    log_level: Box<str>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: NonZeroU16::new(8080).unwrap(),
            log_level: Box::from("debug"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = AppConfig::default();
    let config_store = stains::config::Store::collect();

    println!("{:?}", config);

    for config_source in config_store.iter() {
        print!("Running config: {}... ", config_source.name());
        std::io::stdout().flush()?;

        config_source.config(&mut config).await?;
        println!("config done.");
        println!("{:?}", config);
    }


    Ok(())
}
