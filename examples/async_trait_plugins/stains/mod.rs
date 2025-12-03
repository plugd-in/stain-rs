//! Will hold our [stain] registries.
//!
//! A good general practice is to create a place to
//! put the registries, and do the implementation elsewhere
//! throughout your project.

use anyhow::Result;
use async_trait::async_trait;
use stain::create_stain;

use crate::AppConfig;

// We need to use `async_trait` here because an implicit
// requirement of `stain` is dyn compatibility.
//
// Alternatively, we could just alter the method signature
// to return a `Box<Pin<Future<Output = Result<()>>>>`.
//
// Personally, I prefer the second option, because macro expansion
// can harm code completion in IDEs.
#[async_trait]
pub(crate) trait Config {
    async fn config(&self, config: &mut AppConfig) -> Result<()>;
}

create_stain! {
    trait Config;

    store: pub(crate) mod config;
}
