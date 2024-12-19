use std::env::current_dir;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::data::FileSize;
use crate::options::{find_and_parse_kerblam_toml, KerblamTomlOptions};
use crate::utils::normalize_path;
use crate::utils::{ask_for, find_dirs, YesNo};

use anyhow::{bail, Result};
use clap::Args;
use indicatif::ProgressBar;

use crate::cli::Executable;

/// Clean non-essential data to save disk space
///
/// This removes:
///     - Output data (in the output directory);
///     - Intermediate data (in the intermediate data directory);
/// and all empty directories that are left behind after deletion.
///
/// Examples:
///     > Delete everything that is not precious or remote
///         kerblam data clean
///
///     > Delete everything that is not precious
///         kerblam data clean --include-remote
///
///     > Skip the confirmation prompt
///         kerblam data clean --yes
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct CleanCommand {
    #[arg(long, short('r'), action)]
    /// Also delete locally present remote files.
    include_remote: bool,
    /// Do not delete output files.
    #[arg(long, action)]
    preserve_output: bool,
    #[arg(long, short('d'), action)]
    /// Do not delete locally present directories.
    keep_dirs: bool,
    #[arg(long, short, action)]
    /// Do not ask for any confirmation.
    yes: bool,
    #[arg(long, action)]
    /// Print files that will be cleaned, but don't delete them.
    dry_run: bool,
}

impl Executable for CleanCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        clean_data(
            config,
            !self.include_remote,
            self.preserve_output,
            self.keep_dirs,
            self.yes,
            self.dry_run,
        )
    }
}

/// Delete a list of files
///
/// This shows a nice progress bar while the deletion is in progress.
fn delete_files(files: Vec<PathBuf>) -> Result<()> {
    let progress = ProgressBar::new(files.len() as u64);

    let mut failures: Vec<(PathBuf, std::io::Error)> = vec![];
    for file in progress.wrap_iter(files.into_iter()) {
        if file.metadata().unwrap().is_file() {
            if let Err(e) = fs::remove_file(&file) {
                failures.push((file, e));
            }
        } else if let Err(e) = fs::remove_dir(&file) {
            failures.push((file, e))
        }
    }

    if !failures.is_empty() {
        bail!(
            "Failed to clean some files:\n {}",
            failures.into_iter().fold(String::new(), |mut previous, x| {
                let _ = writeln!(
                    previous,
                    "\t- {}: {}",
                    normalize_path(x.0.strip_prefix(current_dir().unwrap()).unwrap())
                        .to_string_lossy(),
                    x.1
                );
                previous
            })
        )
    };

    Ok(())
}

fn clean_data(
    config: KerblamTomlOptions,
    keep_remote: bool,
    keep_output: bool,
    keep_dirs: bool,
    skip_confirm: bool,
    dry_run: bool,
) -> Result<()> {
    let cleanable_files = config.volatile_files(!keep_output, true);
    let remote_files: Vec<PathBuf> = config
        .remote_files()
        .into_iter()
        .map(|remote| remote.path)
        .collect();

    // Filter out the remote files if we so say
    let cleanable_files: Vec<PathBuf> = if keep_remote {
        cleanable_files
            .into_iter()
            .filter(|x| {
                log::debug!("test: {:?}", x);
                remote_files.iter().all(|remote| x != remote)
            })
            .collect()
    } else {
        cleanable_files
    };

    log::debug!("Files to clean: {:?}", cleanable_files);

    if dry_run {
        let current_dir = current_dir().unwrap();
        let cleanable: Vec<String> = cleanable_files
            .clone()
            .into_iter()
            .map(|x| x.strip_prefix(&current_dir).unwrap().to_owned())
            .map(|x| x.into_os_string().into_string().unwrap())
            .collect();
        println!("Files to clean:\n{}", cleanable.join(" \n"));
        return Ok(());
    }

    if !cleanable_files.is_empty() {
        let question = format!(
            "🧹 About to delete {} files ({}). Continue?",
            cleanable_files.len(),
            &cleanable_files
                .iter()
                .cloned()
                .map(|x| x.try_into().unwrap())
                .sum::<FileSize>()
        );

        if skip_confirm {
            delete_files(cleanable_files.clone())?
        } else {
            match ask_for::<YesNo>(question.as_str()) {
                YesNo::Yes => delete_files(cleanable_files.clone())?,
                YesNo::No => {
                    bail!("Aborted!");
                }
            };
        }
    }

    // After we cleanup the files, we can cleanup the directories
    if keep_dirs {
        if cleanable_files.is_empty() {
            println!("✨ Nothing to clean!")
        }
        return Ok(());
    }

    // A tiny utility to get rid of filter paths that overlap
    fn remove_useless_filters(target: &Path, filters: Vec<PathBuf>) -> Vec<PathBuf> {
        filters
            .into_iter()
            .filter(|x| !target.starts_with(x))
            .collect()
    }

    let dirs = [
        find_dirs(
            config.output_data_dir(),
            Some(remove_useless_filters(
                config.output_data_dir().as_ref(),
                vec![config.input_data_dir(), config.intermediate_data_dir()],
            )),
        ),
        find_dirs(
            config.intermediate_data_dir(),
            Some(remove_useless_filters(
                config.intermediate_data_dir().as_ref(),
                vec![config.input_data_dir(), config.output_data_dir()],
            )),
        ),
    ]
    .concat();

    // Remove the root directories we DO NOT want to clean.
    let mut dirs: Vec<PathBuf> = dirs
        .into_iter()
        .filter(|x| *x != config.output_data_dir() && *x != config.intermediate_data_dir())
        .collect();

    // We need to sort the dirs from deepest to shallowest in order to
    // delete them in order, or else `delete_files` just dies.
    dirs.sort_unstable_by_key(|i| i.ancestors().count());
    dirs.reverse();
    log::debug!("Dirs to clean: {:?}", dirs);

    if !dirs.is_empty() {
        println!("🧹 Removing empty directories left behind...");
        // This dies if the directory is not empty. So it's generally safe
        // even if some bug introduces an error here.
        delete_files(dirs)?;
    } else if cleanable_files.is_empty() {
        println!("✨ Nothing to clean!")
    }

    Ok(())
}
