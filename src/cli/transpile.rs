use std::{path::PathBuf, process::ExitCode, str::FromStr};

use anyhow::Result;
use clap::Parser;
use dal_core::{
    manifest::{Manifest, WritableManifest},
    polyfill::Polyfill,
    transpiler::Transpiler,
};
use std::time::Instant;
use url::Url;

use super::{DEFAULT_MANIFEST_PATH, DEFAULT_POLYFILL_URL};

/// Transpile luau files into lua files
#[derive(Debug, Clone, Parser)]
pub struct TranspileCommand {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl TranspileCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let process_start_time = Instant::now();

        let manifest = Manifest::from_file(DEFAULT_MANIFEST_PATH).await?;
        let polyfill = Polyfill::new(&Url::from_str(DEFAULT_POLYFILL_URL)?).await?;
        let mut transpiler = Transpiler::default();
        transpiler = transpiler.with_manifest(&manifest);
        transpiler = transpiler.with_polyfill(polyfill, None);

        transpiler
            .process(
                manifest.require_input(self.input)?,
                manifest.require_output(self.output)?,
            )
            .await?;

        // let mut output_iter = output_files.iter();
        // if let Some(first_output) = output_iter.next() {
        //     let extension = if let Some(extension) = manifest.extension() {
        //         extension.to_owned()
        //     } else {
        //         first_output
        //             .extension()
        //             .ok_or_else(|| anyhow!("Failed to get extension from output file."))?
        //             .to_string_lossy()
        //             .into_owned()
        //     };

        //     if let Some(module_path) = first_output.parent()
        //         .map(|parent|
        //             parent
        //                 .join(DEFAULT_INJECTED_POLYFILL_NAME)
        //                 .with_extension(extension)
        //         )
        //     {
        //         let injector = Injector::new(
        //             module_path,
        //             polyfill.globals().exports().to_owned(),
        //             manifest.target_version().to_lua_version(),
        //             polyfill.removes().to_owned()
        //         );

        //         for source_path in output_iter {
        //             injector.inject(source_path).await?
        //         }
        //     }
        // }

        let process_duration = durationfmt::to_string(process_start_time.elapsed());

        println!("Successfully transpiled in {}", process_duration);

        return Ok(ExitCode::SUCCESS);
    }
}
