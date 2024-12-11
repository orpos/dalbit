use std::{path::Path, process::ExitCode};

use anyhow::{anyhow, Result};
use clap::Parser;
use kaledis_dalbit::manifest::{Manifest, WritableManifest};

use crate::cli::DEFAULT_MANIFEST_PATH;

/// Initialize dalbit manifest file
#[derive(Debug, Clone, Parser)]
pub struct InitCommand {}

impl InitCommand {
    pub async fn run(self) -> Result<ExitCode> {
        if Path::new(DEFAULT_MANIFEST_PATH).exists() {
            return Err(anyhow!("Manifest has already been initialized"));
        } else {
            let manifest = Manifest::default();
            manifest.write(DEFAULT_MANIFEST_PATH).await?;

            println!("Initialized dalbit manifest");
        }

        return Ok(ExitCode::SUCCESS);
    }
}
