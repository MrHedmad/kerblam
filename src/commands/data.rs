use std::borrow::Cow;
use std::fmt::Display;
use std::fs::{self, create_dir_all};
use std::iter::Sum;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::options::KerblamTomlOptions;

use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use reqwest::blocking::Client;
use walkdir;

#[derive(Debug, Clone)]
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
        let mut res_size = self.size.clone();

        // I want to reduce the size only if the number is greater than 1
        // of the next size
        while res_size > 1024 {
            symbol += 1;
            res_size = res_size / 1024;
            if symbol > 4 {
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

fn find_files(inspected_path: impl AsRef<Path>, filters: Option<Vec<PathBuf>>) -> Vec<PathBuf> {
    let inspected_path = inspected_path.as_ref();

    if let Some(filters) = filters {
        walkdir::WalkDir::new(inspected_path)
            .into_iter()
            .filter_map(|i| i.ok())
            .filter(|x| {
                let mut p = true;
                for path in filters.clone() {
                    if x.path().starts_with(path) {
                        p = false;
                    }
                }
                p
            })
            .filter(|path| path.metadata().unwrap().is_file())
            .map(|x| x.path().to_owned())
            .collect()
    } else {
        walkdir::WalkDir::new(inspected_path)
            .into_iter()
            .filter_map(|i| i.ok())
            .filter(|path| path.metadata().unwrap().is_file())
            .map(|x| x.path().to_owned())
            .collect()
    }
}

pub fn get_data_status(
    config: KerblamTomlOptions,
    inspected_path: impl AsRef<Path>,
) -> Result<DataStatus> {
    let inspected_path = inspected_path.as_ref();

    let input_files = find_files(inspected_path.join("in"), None);
    log::debug!("Input files: {:?}", input_files);
    let output_files = find_files(inspected_path.join("out"), None);
    log::debug!("Output files: {:?}", output_files);
    let temp_files = find_files(
        inspected_path,
        Some(vec![inspected_path.join("in"), inspected_path.join("out")]),
    );
    log::debug!("Temp files: {:?}", output_files);

    // We need to find out what paths are remote
    let specified_remote_files: Vec<PathBuf> =
        config.remote_files().into_iter().map(|x| x.path).collect();
    log::debug!("Remote files: {specified_remote_files:?}");
    let undownloaded_files: Vec<PathBuf> = specified_remote_files
        .clone()
        .into_iter()
        .filter(|remote_path| {
            // If any of the local paths end with the remote path, then
            // the file is not undownloaded.
            !input_files.iter().any(|x| x.ends_with(remote_path))
        })
        .collect();
    let remote_files: Vec<PathBuf> = specified_remote_files
        .clone()
        .into_iter()
        .filter(|remote_path| {
            // Same as above, but inversed
            input_files.iter().any(|x| x == remote_path)
        })
        .collect();
    log::debug!("Undownloded files: {undownloaded_files:?}");
    log::debug!("Downloaded files: {remote_files:?}");

    Ok(DataStatus {
        temp_data: unsafe_path_filesize_conversion(&temp_files),
        input_data: unsafe_path_filesize_conversion(&input_files),
        output_data: unsafe_path_filesize_conversion(&output_files),
        remote_data: unsafe_path_filesize_conversion(&remote_files),
        cleanable_data: unsafe_path_filesize_conversion(
            &[remote_files, output_files, temp_files].concat(),
        ),
        not_local: undownloaded_files.len() as u64,
    })
}

fn unsafe_path_filesize_conversion(items: &Vec<PathBuf>) -> Vec<FileSize> {
    items
        .to_owned()
        .into_iter()
        .map(|x| x.try_into().unwrap())
        .collect()
}

pub fn fetch_remote_data(config: KerblamTomlOptions) -> Result<()> {
    let remote_files = config.remote_files();

    if remote_files.is_empty() {
        println!("No remote files to fetch!");
        return Ok(());
    }

    let mut success = true;

    let client = Client::new();

    let spinner_bar_style =
        ProgressStyle::with_template("⬇️  [{binary_bytes_per_sec}] {msg} {spinner} ({elapsed})")
            .unwrap();
    let bar_style = ProgressStyle::with_template(
        "⬇️  [{binary_bytes_per_sec}] {msg} {wide_bar} {eta} ({elapsed})",
    )
    .unwrap();

    for file in remote_files {
        // Stop if the file is already there
        let filename = file.path.file_name().unwrap().to_string_lossy();

        if file.path.exists() {
            println!("✅ file {} exists!", filename);
            continue;
        }

        let created_msg = format!("Created {}!", filename);
        let bar_msg = format!("Fetching {}", filename);

        let mut result = match client.get(file.url).send() {
            Ok(res) => res,
            Err(e) => {
                println!("Failed to fetch {}! {}", filename, e);
                success = false;
                continue;
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
            eprintln!(
                "❌ Failed retrieving {}! {} ({})",
                filename,
                result.status().canonical_reason().unwrap(),
                result.status().as_str()
            );
            success = false;
            continue;
        }

        // Create the container dir and open a file connection
        let _ = create_dir_all(file.path.parent().unwrap());
        let writer = match fs::File::create(file.path) {
            Ok(f) => f,
            Err(e) => {
                progress.abandon_with_message(Cow::from(format!(
                    " ❌ Failed to create output file! {e:?}"
                )));
                success = false;
                continue;
            }
        };

        match result.copy_to(&mut progress.wrap_write(writer)) {
            Ok(_) => progress.finish_using_style(),
            Err(e) => {
                println!(" ❌ Failed to write to output buffer: {e}");
                success = false;
            }
        };
    }

    if success {
        Ok(())
    } else {
        Err(anyhow!("Something failed to be retrieved."))
    }
}
