use filetime::{set_file_mtime, FileTime};
use similar::{ChangeTag, TextDiff};
use std::env::set_var;
use std::fmt::{Debug, Write};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::slice::Iter;
use tempfile::TempDir;

// All of these functions are panic-heavy

pub fn init_log() {
    set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn assert_ok<T, E>(value: Result<T, E>)
where
    E: Debug,
    T: Debug,
{
    if value.is_ok() {
        assert!(true);
        return;
    }
    if value.is_err() {
        eprintln!("{:?}", value.unwrap_err());
        assert!(false);
    }
}

/// Util to represent a file in the FS
pub struct File {
    path: PathBuf,
    content: String,
}

#[allow(dead_code)]
pub enum FileError {
    NotFound,
    Changed { diff: String },
}

#[allow(dead_code)]
fn compute_diff(old: &str, new: &str) -> String {
    let mut result = String::new();
    let diff = TextDiff::from_lines(old, new);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        writeln!(result, "{}{}", sign, change).unwrap(); // writing to String never fails
    }

    result
}

impl File {
    /// Write the content of the file to disk
    pub fn write(&self) {
        create_dir_all(self.path.parent().unwrap()).expect("Expected to be able to create dirs");

        write(&self.path, &self.content).expect("Failed to write file!");
    }

    /// Does this file exist?
    #[allow(dead_code)]
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Was this file modified in any way from the original content?
    #[allow(dead_code)]
    pub fn check_modified_content(&self) -> Result<(), FileError> {
        self.check_content(&self.content)
    }

    /// Check the content of this file for the expected content
    #[allow(dead_code)]
    pub fn check_content(&self, expected: &str) -> Result<(), FileError> {
        if !self.exists() {
            return Err(FileError::NotFound);
        };

        let content = read_to_string(&self.path).unwrap();
        if content != expected {
            let diff = compute_diff(expected, &content);
            return Err(FileError::Changed { diff });
        }

        Ok(())
    }

    /// Convenience that builds the File from a &str path
    pub fn new(path: &str, content: &str) -> Self {
        Self {
            path: PathBuf::from(path),
            content: content.into(),
        }
    }
}

/// Setup a temporary working directory with a series of files
///
/// This function returns a TempDir. When it gets destroyed, the
/// workdir is deleted as well.
pub fn setup_workdir(files: Iter<File>) -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    let new_files: Vec<File> = files
        .map(|f| File {
            path: temp_dir.path().join(&f.path),
            content: f.content.clone(),
        })
        .collect();

    for file in new_files {
        file.write();
    }

    temp_dir
}
