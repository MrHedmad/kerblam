use std::borrow::Cow;
use std::env::current_dir;
use std::fmt::Write;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

use crate::options::{KerblamTomlOptions, RemoteFile};
use crate::utils::normalize_path;
use crate::utils::{ask_for, find_dirs, run_command, YesNo};

use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use reqwest::blocking::Client;
use url::Url;

pub mod data_description;

use data_description::FileSize;

/// Fetch the remote files specified in the options
///
/// Shows a pretty download bar for each, and gracefully handles files
/// that are already there to begin with.
pub fn fetch_remote_data(config: KerblamTomlOptions) -> Result<()> {
    // I work with this instead of undownloaded_files() to show a message
    // when the file is already there, but it's a choice I guess.
    let remote_files = config.remote_files();

    if remote_files.is_empty() {
        println!("No remote files to fetch!");
        return Ok(());
    }

    // Send a message reminding the user that there are some unfetcheable remote files
    let non_fetcheable: Vec<&RemoteFile> =
        remote_files.iter().filter(|x| x.url.is_none()).collect();

    if !non_fetcheable.is_empty() {
        if non_fetcheable.len() <= 5 {
            let names: Vec<&str> = non_fetcheable
                .into_iter()
                .map(|x| x.to_owned().path.to_str().unwrap())
                .collect();
            if names.len() == 1 {
                eprintln!(
                    "❓ There is one remote file that Kerblam! cannot fetch: {}",
                    names.join(", ")
                );
            } else {
                eprintln!(
                    "❓ There are {} remote files that Kerblam! cannot fetch: {}",
                    names.len(),
                    names.join(", ")
                );
            }
        } else {
            eprintln!(
                "❓ There are {} remote files that Kerblam! cannot fetch",
                non_fetcheable.len(),
            );
        }
    }

    let remote_files: Vec<RemoteFile> = remote_files
        .into_iter()
        .filter(|x| x.url.is_some())
        .collect();

    log::debug!("Fetching files: {:?}", remote_files);

    // Check if any remote files will be saved somewhere else than the
    // input data dir. If so, warn the user before continuing.
    let data_dir = config.input_data_dir();
    let non_canonical_files: Vec<&RemoteFile> = remote_files
        .iter()
        .filter(|x| !x.path.starts_with(&data_dir))
        .collect();
    if !non_canonical_files.is_empty() {
        let msg = non_canonical_files
            .into_iter()
            .map(|x| x.path.clone().into_os_string().into_string().unwrap())
            .map(|x| format!("\t- {}", x))
            .reduce(|x, y| format!("{}\n{}", x, y))
            .unwrap();
        eprintln!(
            "⚠️  Some target paths are not inside the input data directory:\n{}",
            msg
        );

        let approve = ask_for::<YesNo>("Continue?");
        if matches!(approve, YesNo::No) {
            return Ok(());
        }
    }

    let mut success = true;
    let client = Client::new();

    for file in remote_files {
        // Stop if the file is already there
        if file.path.exists() {
            let filename = file.path.file_name().unwrap().to_string_lossy();
            println!("✅ file {} exists!", filename);
            continue;
        }

        if let Err(msg) = fetch_remote_file(
            &client,
            file.url.expect("This is expected due to above filtering"),
            file.path,
        ) {
            eprintln!("{}", msg);
            success = false;
        }
    }

    if !success {
        bail!("Something failed to be retrieved.")
    }

    Ok(())
}

/// Fetch a remote file `url` and save it to `target` using `client`
///
/// The fetching is nicely wrapped up into a progress bar that shows
/// the download progress.
/// If the download fails, the progress bar is cleaned up an this function
/// returns the error.
fn fetch_remote_file(client: &Client, url: Url, target: PathBuf) -> Result<()> {
    let spinner_bar_style =
        ProgressStyle::with_template("⬇️  [{binary_bytes_per_sec}] {msg} {spinner} ({elapsed})")
            .unwrap();
    let bar_style = ProgressStyle::with_template(
        "⬇️  [{binary_bytes_per_sec}] {msg} {wide_bar} {eta} ({elapsed})",
    )
    .unwrap();
    let filename = target.file_name().unwrap().to_string_lossy();
    let created_msg = format!("Created {}!", filename);
    let bar_msg = format!("Fetching {}", filename);

    let mut result = match client.get(url).send() {
        Ok(res) => res,
        Err(e) => {
            bail!("Failed to fetch {}! {}", target.to_string_lossy(), e);
        }
    };
    // See if we have the content size
    let size = result
        .headers()
        .get("content-length")
        .and_then(|val| val.to_str().unwrap().parse::<u64>().ok());

    let progress = match size {
        None => ProgressBar::new_spinner()
            .with_style(spinner_bar_style.clone())
            .with_message(bar_msg)
            .with_finish(ProgressFinish::WithMessage(Cow::from(created_msg))),
        Some(val) => ProgressBar::new(val)
            .with_style(bar_style.clone())
            .with_message(bar_msg)
            .with_finish(ProgressFinish::WithMessage(Cow::from(created_msg))),
    };

    if !result.status().is_success() {
        progress.finish_and_clear();
        bail!(
            "❌ Failed retrieving {}! {} ({})",
            filename,
            result.status().canonical_reason().unwrap(),
            result.status().as_str()
        );
    }

    // Create the container dir and open a file connection
    let _ = create_dir_all(target.parent().unwrap());
    let writer = match fs::File::create(target) {
        Ok(f) => f,
        Err(e) => {
            // We are failing but we still clear the bar as normal
            progress.finish_and_clear();
            bail!("Failed to create output file! {e:?}")
        }
    };

    match result.copy_to(&mut progress.wrap_write(writer)) {
        Ok(_) => progress.finish_using_style(),
        Err(e) => {
            bail!(" ❌ Failed to write to output buffer: {e}");
        }
    };

    Ok(())
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
