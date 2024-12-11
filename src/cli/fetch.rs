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
        


        return Ok(ExitCode::SUCCESS);
    }
}
