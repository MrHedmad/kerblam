use core::iter::Sum;
use std::fmt::Display;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::options::KerblamTomlOptions;

#[derive(Debug, Clone)]
/// Wrapper to interpret a usize as a File size.
pub struct FileSize {
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

impl TryFrom<KerblamTomlOptions> for DataStatus {
    fn try_from(value: KerblamTomlOptions) -> Result<Self, Self::Error> {
        let input_files = value.input_files();
        let output_files = value.output_files();
        let int_files = value.intermediate_files();
        log::debug!("Output files: {:?}", output_files);
        log::debug!("Input files: {:?}", input_files);
        log::debug!("Temp files: {:?}", int_files);

        let undownloaded_files: Vec<PathBuf> = value
            .undownloaded_files()
            .into_iter()
            .map(Into::into)
            .collect();
        let remote_files: Vec<PathBuf> = value
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
            cleanable_data: unsafe_path_filesize_conversion(&value.volatile_files()),
            not_local: undownloaded_files.len() as u64,
        })
    }

    type Error = anyhow::Error;
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
