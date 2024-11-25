use crate::cli::Executable;
use crate::options::find_and_parse_kerblam_toml;
use clap::Args;
use std::borrow::Cow;
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

use crate::options::{KerblamTomlOptions, RemoteFile};
use crate::utils::{ask_for, YesNo};

use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use reqwest::blocking::Client;
use url::Url;

/// Fetch remote data and save it locally
///
/// You can specify data to fetch in the kerblam.toml file, like so:
///
/// [data.remote]
/// "url to be fetched" = "target file"
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct FetchCommand {}

impl Executable for FetchCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        fetch_remote_data(config)
    }
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
