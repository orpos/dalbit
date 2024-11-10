use std::process::ExitCode;

mod cli;

use cli::Dal;
pub use dal_core;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    match Dal::new().run().await {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}
