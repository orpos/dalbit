use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::Parser;
use dal_core::manifest::{Manifest, DEFAULT_MANIFEST_PATH};

/// Transpile luau files into lua files
#[derive(Debug, Clone, Parser)]
pub struct TranspileCommand {
    #[arg(long)]
    input: Vec<PathBuf>,
    #[arg(long)]
    output: PathBuf,
}

impl TranspileCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let manifest = Manifest::from_file(DEFAULT_MANIFEST_PATH).await?;

        dal_core::process(manifest, self.input, self.output).await?;

        return Ok(ExitCode::SUCCESS);
    }
}
