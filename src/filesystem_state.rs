use std::collections::{HashMap, HashSet};
use std::env::current_dir;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use log;

/// Represents a file's metadata in our snapshot
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub is_dir: bool,
}

/// Represents a snapshot of the filesystem at a point in time
#[derive(Debug, Clone)]
pub struct FilesystemState {
    pub files: HashMap<PathBuf, FileInfo>,
    pub root_dir: PathBuf,
    pub snapshot_time: SystemTime,
}

/// Represents the kind of change that occurred to a file
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Deleted,
    Modified,
    Accessed,
}

/// Represents a change to a specific file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub old_info: Option<FileInfo>,
    pub new_info: Option<FileInfo>,
}

/// Represents all changes between two filesystem states
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilesystemDiff {
    pub changes: Vec<FileChange>,
    pub from_time: SystemTime,
    pub to_time: SystemTime,
}

impl FilesystemState {
    /// Creates a new filesystem state by scanning from the given root directory
    pub fn new<P: AsRef<Path>>(root_dir: P) -> io::Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        let snapshot_time = SystemTime::now();
        let mut files = HashMap::new();

        Self::scan_directory(&root_dir, &mut files)?;

        Ok(FilesystemState {
            files,
            root_dir,
            snapshot_time,
        })
    }

    /// Recursively scans a directory to build the filesystem state
    fn scan_directory(dir: &Path, files: &mut HashMap<PathBuf, FileInfo>) -> io::Result<()> {
        log::debug!("Scanning directory: {:?}", dir);
        if !dir.exists() {
            // This dir does not exit - treat it as if it was empty.
            return Ok(());
        }

        if !dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not a directory",
            ));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            files.insert(path.clone(), Self::create_file_info(&path, &metadata)?);

            if metadata.is_dir() {
                Self::scan_directory(&path, files)?;
            }
        }

        Ok(())
    }

    /// Creates a FileInfo struct from a path and its metadata
    fn create_file_info(path: &Path, metadata: &Metadata) -> io::Result<FileInfo> {
        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);

        Ok(FileInfo {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified,
            accessed,
            is_dir: metadata.is_dir(),
        })
    }

    /// Compares this state with another state to produce a diff
    pub fn diff(&self, other: &FilesystemState) -> FilesystemDiff {
        let mut changes = Vec::new();

        // Find files that exist in both states, were deleted, or were created
        let self_paths: HashSet<&PathBuf> = self.files.keys().collect();
        let other_paths: HashSet<&PathBuf> = other.files.keys().collect();

        // Check for deleted files (exist in self but not in other)
        for path in self_paths.difference(&other_paths) {
            if let Some(file_info) = self.files.get(*path) {
                changes.push(FileChange {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Deleted,
                    old_info: Some(file_info.clone()),
                    new_info: None,
                });
            }
        }

        // Check for created files (exist in other but not in self)
        for path in other_paths.difference(&self_paths) {
            if let Some(file_info) = other.files.get(*path) {
                changes.push(FileChange {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Created,
                    old_info: None,
                    new_info: Some(file_info.clone()),
                });
            }
        }

        // Check for modified files (exist in both but have different attributes)
        for path in self_paths.intersection(&other_paths) {
            let self_info = &self.files[*path];
            let other_info = &other.files[*path];

            // Check if file was modified (size or modification time changed)
            if self_info.size != other_info.size || self_info.modified != other_info.modified {
                changes.push(FileChange {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Modified,
                    old_info: Some(self_info.clone()),
                    new_info: Some(other_info.clone()),
                });
            }
            // Check if file was accessed but not modified
            else if self_info.accessed != other_info.accessed {
                changes.push(FileChange {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Accessed,
                    old_info: Some(self_info.clone()),
                    new_info: Some(other_info.clone()),
                });
            }
        }

        FilesystemDiff {
            changes,
            from_time: self.snapshot_time,
            to_time: other.snapshot_time,
        }
    }
}

/// Function to compare two filesystem states and produce a diff
pub fn compare_filesystem_states(
    before: &FilesystemState,
    after: &FilesystemState,
) -> FilesystemDiff {
    before.diff(after)
}

/// Generates a human-readable tree-like summary of filesystem changes
pub fn generate_tree_summary(diffs: &[FilesystemDiff]) -> String {
    let here = current_dir().unwrap();
    if diffs.is_empty() {
        return "No changes detected.".to_string();
    }

    // Maps paths to their change types
    let mut path_changes: HashMap<PathBuf, Vec<(ChangeType, SystemTime)>> = HashMap::new();

    // Collect all unique paths and their associated changes across all diffs
    for diff in diffs {
        for change in &diff.changes {
            path_changes
                .entry(change.path.clone().strip_prefix(&here).unwrap().into())
                .or_default()
                .push((change.change_type.clone(), diff.to_time));
        }
    }

    // Build directory structure
    let mut dir_tree: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut all_dirs = HashSet::new();

    // Find all involved directories
    for path in path_changes.keys() {
        let mut current = path.clone();
        while let Some(parent) = current.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }

            dir_tree
                .entry(parent.to_path_buf())
                .or_default()
                .insert(current.clone());

            all_dirs.insert(parent.to_path_buf());
            current = parent.to_path_buf();
        }
    }

    // Find root(s) - directories that aren't contained in other directories
    let mut roots: Vec<PathBuf> = vec![];
    for dir in &all_dirs {
        if !all_dirs
            .iter()
            .any(|other| other != dir && dir.starts_with(other))
        {
            roots.push(dir.clone());
        }
    }

    roots.sort();

    // Build the tree string
    let mut result = String::new();
    result.push_str("File System Changes:\n");

    for root in roots {
        result.push_str(&format!("{}\n", root.display()));
        build_tree_string(&root, &dir_tree, &path_changes, &mut result, "");
    }

    result
}

/// Helper function to recursively build the tree string representation
fn build_tree_string(
    dir: &Path,
    dir_tree: &HashMap<PathBuf, HashSet<PathBuf>>,
    path_changes: &HashMap<PathBuf, Vec<(ChangeType, SystemTime)>>,
    result: &mut String,
    prefix: &str,
) {
    let mut entries: Vec<PathBuf> = if let Some(children) = dir_tree.get(dir) {
        children.iter().cloned().collect()
    } else {
        vec![]
    };

    // Sort entries for consistent output
    entries.sort();

    // Process all but the last entry
    let count = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let is_last = i == count - 1;
        let entry_prefix = if is_last { "└── " } else { "├── " };
        let next_prefix = if is_last { "    " } else { "│   " };

        // Add change indicators with emojis
        let mut change_info = String::new();
        if let Some(changes) = path_changes.get(&entry) {
            // Sort changes by time to show most recent first
            let mut sorted_changes = changes.clone();
            sorted_changes.sort_by(|a, b| b.1.cmp(&a.1));

            for (change_type, _) in sorted_changes.iter().take(1) {
                // Show only most recent change
                let emoji = match change_type {
                    ChangeType::Created => "✨ ",  // Sparkles for created
                    ChangeType::Deleted => "🗑️ ",  // Trash for deleted
                    ChangeType::Modified => "📝 ", // Pencil for modified
                    ChangeType::Accessed => "👁️ ", // Eye for accessed
                };
                change_info.push_str(emoji);
            }
        }

        let display_name = entry
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.to_string_lossy().to_string());

        result.push_str(&format!(
            "{}{}{}{}\n",
            prefix, entry_prefix, change_info, display_name
        ));

        // Recurse for directories
        if dir_tree.contains_key(&entry) {
            build_tree_string(
                &entry,
                dir_tree,
                path_changes,
                result,
                &format!("{}{}", prefix, next_prefix),
            );
        }
    }
}
