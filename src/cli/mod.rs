use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod init;
mod transpile;

use init::InitCommand;
use transpile::TranspileCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Transpile(TranspileCommand),
    Init(InitCommand),
}

/// Transpile Luau scripts
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Dal {
    #[clap(subcommand)]
    subcommand: CliSubcommand,
}

impl Dal {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn run(self) -> Result<ExitCode> {
        match self.subcommand {
            CliSubcommand::Transpile(cmd) => cmd.run().await,
            CliSubcommand::Init(cmd) => cmd.run().await,
        }
    }
}
