use std::env::current_dir;
use std::fmt::Write;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

use crate::options::KerblamTomlOptions;
use crate::utils::normalize_path;
use crate::utils::{ask_for, find_dirs, run_command, YesNo};

use anyhow::{bail, Result};
use indicatif::ProgressBar;

pub mod data_description;
pub mod fetch;

use data_description::FileSize;

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

pub fn clean_data(
    config: KerblamTomlOptions,
    keep_remote: bool,
    keep_dirs: bool,
    skip_confirm: bool,
    dry_run: bool,
) -> Result<()> {
    let cleanable_files = config.volatile_files();
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

pub fn package_data_to_archive(
    config: KerblamTomlOptions,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let output_path = output_path.as_ref();
    // This is to render relative paths not relative.
    let output_path = current_dir()?.join(output_path);

    let precious_files = config.precious_files();

    let files_to_package: Vec<PathBuf> = [precious_files, config.output_files()]
        .concat()
        .into_iter()
        .filter(|x| x.is_file())
        .collect();

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
