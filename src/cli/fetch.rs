use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use dalbit_core::manifest::{Manifest, WritableManifest};

use super::DEFAULT_MANIFEST_PATH;

/// Fetch dalbit polyfills
#[derive(Debug, Clone, Parser)]
pub struct FetchCommand {}

impl FetchCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let manifest = Manifest::from_file(DEFAULT_MANIFEST_PATH).await?;
        let polyfill_cache = manifest.polyfill().cache().await?;
        polyfill_cache.fetch()?;

        // TO-DO: Is fetched polyfill already latest version?

        return Ok(ExitCode::SUCCESS);
    }
}
