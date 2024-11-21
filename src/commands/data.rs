use std::borrow::Cow;
use std::env::current_dir;
use std::fmt::{Display, Write};
use std::fs::{self, create_dir_all};
use std::iter::Sum;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::options::{KerblamTomlOptions, RemoteFile};
use crate::utils::normalize_path;
use crate::utils::{ask_for, find_dirs, run_command, YesNo};

use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use reqwest::blocking::Client;
use url::Url;

#[derive(Debug, Clone)]
/// Wrapper to interpret a usize as a File size.
struct FileSize {
    size: usize,
}

impl Copy for FileSize {}

impl TryFrom<PathBuf> for FileSize {
    type Error = anyhow::Error;
    fn try_from(value: PathBuf) -> std::result::Result<Self, Self::Error> {
        let meta = fs::metadata(value)?;

        Ok(FileSize {
            size: meta.size() as usize,
        })
    }
}

impl Sum<FileSize> for FileSize {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total: usize = 0;
        for item in iter {
            total += item.size;
        }

        FileSize { size: total }
    }
}

pub struct DataStatus {
    // We don't care to the individuality of the
    // files, so just store all of their values
    temp_data: Vec<FileSize>,
    input_data: Vec<FileSize>,
    output_data: Vec<FileSize>,
    remote_data: Vec<FileSize>,
    cleanable_data: Vec<FileSize>,
    not_local: u64,
}

impl DataStatus {
    /// Get the total size of the data
    fn total_local_size(&self) -> FileSize {
        let all_data = [
            self.temp_data.clone(),
            self.input_data.clone(),
            self.output_data.clone(),
        ]
        .concat();

        all_data.into_iter().sum()
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut symbol: i8 = 0;
        let mut res_size = self.size;

        // I want to reduce the size only if the number is greater than 1
        // of the next size.
        // These are "traditional" bytes, so it's 1024 for each filesize
        while res_size > 1024 {
            symbol += 1;
            res_size /= 1024;
            if symbol > 4 {
                // If we get to petabytes, it's best if we stop now.
                break;
            };
        }

        let symbol = match symbol {
            0 => "B",
            1 => "KiB",
            2 => "MiB",
            3 => "GiB",
            4 => "TiB",
            _ => "PiB", // I doubt we need much more that that.
        };

        write!(f, "{} {}", res_size, symbol)
    }
}

impl Display for DataStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut concat: Vec<String> = Vec::with_capacity(14);

        concat.push(format!(
            "./data\t{} [{}]",
            self.temp_data.clone().into_iter().sum::<FileSize>(),
            self.temp_data.len()
        ));

        concat.push(format!(
            "└── in\t{} [{}]",
            self.input_data.clone().into_iter().sum::<FileSize>(),
            self.input_data.len()
        ));

        concat.push(format!(
            "└── out\t{} [{}]",
            self.output_data.clone().into_iter().sum::<FileSize>(),
            self.output_data.len()
        ));

        concat.push("──────────────────────".into());
        concat.push(format!(
            "Total\t{} [{}]",
            self.total_local_size(),
            self.output_data.len() + self.temp_data.len() + self.input_data.len()
        ));

        concat.push(format!(
            "└── cleanup\t{} [{}] ({:.2}%)",
            self.cleanable_data.clone().into_iter().sum::<FileSize>(),
            self.cleanable_data.len(),
            (self
                .cleanable_data
                .clone()
                .into_iter()
                .sum::<FileSize>()
                .size as f64
                / self.total_local_size().size as f64)
                * 100.
        ));

        concat.push(format!(
            "└── remote\t{} [{}]",
            self.remote_data.clone().into_iter().sum::<FileSize>(),
            self.remote_data.len()
        ));

        if self.not_local != 1 {
            concat.push(format!("There are {} undownloaded files.", self.not_local));
        } else {
            concat.push("There is one undownloaded file.".to_string());
        };

        write!(f, "{}", concat.join("\n"))
    }
}

pub fn get_data_status(config: KerblamTomlOptions) -> Result<DataStatus> {
    let input_files = config.input_files();
    let output_files = config.output_files();
    let int_files = config.intermediate_files();
    log::debug!("Output files: {:?}", output_files);
    log::debug!("Input files: {:?}", input_files);
    log::debug!("Temp files: {:?}", int_files);

    let undownloaded_files: Vec<PathBuf> = config
        .undownloaded_files()
        .into_iter()
        .map(Into::into)
        .collect();
    let remote_files: Vec<PathBuf> = config
        .downloaded_files()
        .into_iter()
        .map(Into::into)
        .collect();
    log::debug!("Undownloded files: {undownloaded_files:?}");
    log::debug!("Downloaded files: {remote_files:?}");

    Ok(DataStatus {
        temp_data: unsafe_path_filesize_conversion(&int_files),
        input_data: unsafe_path_filesize_conversion(&input_files),
        output_data: unsafe_path_filesize_conversion(&output_files),
        remote_data: unsafe_path_filesize_conversion(&remote_files),
        cleanable_data: unsafe_path_filesize_conversion(&config.volatile_files()),
        not_local: undownloaded_files.len() as u64,
    })
}

/// Convert a vector of paths to a vector of file sizes.
///
/// This is unsafe as the path might not exist, so there is a dangerous
/// 'unwrap' in here. Use it only when it's fairly certain that the file
/// is there.
fn unsafe_path_filesize_conversion(items: &[PathBuf]) -> Vec<FileSize> {
    items
        .iter()
        .cloned()
        .map(|x| x.try_into().unwrap())
        .collect()
}

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
    let non_fetcheable: Vec<&RemoteFile> = remote_files
        .iter()
        .filter_map(|x| {
            if x.path.to_str().unwrap() == "_" {
                Some(x)
            } else {
                None
            }
        })
        .collect();

    if !non_fetcheable.is_empty() {
        if non_fetcheable.len() <= 5 {
            let names: Vec<&str> = non_fetcheable
                .into_iter()
                .map(|x| x.to_owned().path.to_str().unwrap())
                .collect();
            eprintln!(
                "There are {} remote files that Kerblam! cannot fetch: {}",
                names.len(),
                names.join(", ")
            );
        } else {
            eprintln!(
                "There are {} remote files that Kerblam! cannot fetch",
                non_fetcheable.len(),
            );
        }
    }

    let remote_files: Vec<RemoteFile> = remote_files
        .into_iter()
        .filter(|x| x.path.to_str().unwrap() != "_")
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

        if let Err(msg) = fetch_remote_file(&client, file.url, file.path) {
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
            unsafe_path_filesize_conversion(&cleanable_files)
                .into_iter()
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
