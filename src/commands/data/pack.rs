use std::env::current_dir;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

use crate::cli::Executable;
use crate::options::find_and_parse_kerblam_toml;
use crate::options::KerblamTomlOptions;
use crate::utils::normalize_path;
use crate::utils::run_command;

use anyhow::Result;
use clap::Args;

/// Pack local data for export to others
///
/// This creates a tarball with the local-only input data and the output
/// data currently present on disk.
///
/// The created file will always be a tar.gz file, regardless of the name
/// you give it.
///
/// Example:
///      kerblam data pack my_last_execution.tar.gz
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct PackCommand {
    /// Output path to store the resulting tar.gz file.
    output_path: Option<PathBuf>,
    /// Do not include input data, only output.
    #[arg(long, short, action)]
    output_only: bool,
}

impl Executable for PackCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        package_data_to_archive(
            config,
            self.output_path
                .unwrap_or(current_dir()?.join("data/data_export.tar.gz")),
            self.output_only,
        )
    }
}

fn package_data_to_archive(
    config: KerblamTomlOptions,
    output_path: impl AsRef<Path>,
    output_only: bool,
) -> Result<()> {
    let output_path = output_path.as_ref();
    // This is to render relative paths not relative.
    let output_path = current_dir()?.join(output_path);

    let precious_files = config.precious_files();

    let files_to_package: Vec<PathBuf> = if output_only {
        config
            .output_files()
            .into_iter()
            .filter(|x| x.is_file())
            .collect()
    } else {
        [precious_files, config.output_files()]
            .concat()
            .into_iter()
            .filter(|x| x.is_file())
            .collect()
    };

    if files_to_package.is_empty() {
        println!("🕸️ Nothing to pack!");
        return Ok(());
    }

    let compression_dir = tempfile::tempdir()?;
    let compression_dir_path = compression_dir.path();

    let root_path = current_dir().unwrap();
    for file in files_to_package {
        println!("➕ Adding {:?}...", normalize_path(file.as_ref()));
        let target_file = compression_dir_path
            .to_path_buf()
            .join(file.strip_prefix(&root_path)?);
        log::debug!("Moving {file:?} to {target_file:?}");
        create_dir_all(target_file.parent().unwrap())?;
        fs::copy(&file, &target_file)?;
    }

    println!("Compressing...");
    run_command(
        compression_dir_path.parent(),
        "tar",
        vec![
            "czf",
            output_path.as_os_str().to_str().unwrap(),
            "-C",
            compression_dir_path.to_str().unwrap(),
            ".",
        ],
    )?;

    drop(compression_dir);

    println!("✨ Done!");
    Ok(())
}
