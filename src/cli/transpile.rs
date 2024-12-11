use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use dalbit_core::{
    manifest::{Manifest, WritableManifest},
    transpile,
};
use std::time::Instant;

use super::DEFAULT_MANIFEST_PATH;

/// Transpile luau files into lua files
#[derive(Debug, Clone, Parser)]
pub struct TranspileCommand {}

impl TranspileCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let process_start_time = Instant::now();

        let manifest = Manifest::from_file(DEFAULT_MANIFEST_PATH).await?;

        transpile::process(manifest).await?;

        let process_duration = durationfmt::to_string(process_start_time.elapsed());

        println!("Successfully transpiled in {}", process_duration);

        return Ok(ExitCode::SUCCESS);
    }
}
