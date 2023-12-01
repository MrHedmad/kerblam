use std::fmt::Display;
use std::fs;
use std::iter::Sum;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::options::KerblamTomlOptions;

use anyhow::Result;
use walkdir;

#[derive(Debug, Clone)]
struct FileSize {
    size: usize,
}

impl Copy for FileSize {}

impl FileSize {
    fn from_path(value: impl AsRef<Path>) -> Result<Self> {
        let value = value.as_ref();

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

struct DataStatus {
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
            "Total\n{} [{}]",
            self.total_local_size(),
            self.output_data.len() + self.temp_data.len() + self.input_data.len()
        ));

        concat.push(format!(
            "└── cleanup\t{} [{}] ({})",
            self.cleanable_data.clone().into_iter().sum::<FileSize>(),
            self.cleanable_data.len(),
            self.total_local_size().size
                / self
                    .cleanable_data
                    .clone()
                    .into_iter()
                    .sum::<FileSize>()
                    .size
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

pub fn get_data_status(
    config: KerblamTomlOptions,
    inspected_path: impl AsRef<Path>,
) -> Result<DataStatus> {
    let inspected_path = inspected_path.as_ref();

    let input_files: Vec<PathBuf> = walkdir::WalkDir::new(inspected_path.join("in"))
        .into_iter()
        .filter_map(|i| i.ok())
        .map(|x| x.path().to_owned())
        .collect();
    let output_files: Vec<PathBuf> = walkdir::WalkDir::new(inspected_path.join("out"))
        .into_iter()
        .filter_map(|i| i.ok())
        .map(|x| x.path().to_owned())
        .collect();
    let temp_files: Vec<PathBuf> = walkdir::WalkDir::new(inspected_path)
        .into_iter()
        .filter_entry(|x| {
            x.path().starts_with(inspected_path.join("in"))
                | x.path().starts_with(inspected_path.join("out"))
        })
        .filter_map(|i| i.ok())
        .map(|x| x.path().to_owned())
        .collect();

    todo!()
}
