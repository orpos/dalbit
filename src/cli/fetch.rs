use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use dal_core::manifest::{Manifest, WritableManifest};

use super::DEFAULT_MANIFEST_PATH;

/// Fetch dal polyfills
#[derive(Debug, Clone, Parser)]
pub struct FetchCommand {}

impl FetchCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let manifest = Manifest::from_file(DEFAULT_MANIFEST_PATH).await?;
        for polyfill in manifest.polyfills() {
            let polyfill_cache = polyfill.cache().await?;
            polyfill_cache.fetch()?;
        }

        return Ok(ExitCode::SUCCESS);
    }
}
