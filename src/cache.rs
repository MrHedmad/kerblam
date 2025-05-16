use chrono::{self, Utc};
use std::env::current_dir;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use homedir::get_my_home;
use serde::{Deserialize, Serialize};

use crate::filesystem_state::FilesystemDiff;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Cache {
    pub last_executed_profile: Option<String>,
    pub run_metadata: Option<Vec<RunMetadata>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RunMetadata {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub exit_code: i32,
    pub pipe_path: PathBuf,
    pub env_path: Option<PathBuf>,
    pub used_env: bool,
    pub modified_files: Vec<FilesystemDiff>,
}

/// Return the cache file for the current directory
///
/// The location of the cache file is dependent on the hash of the path
/// of the current directory, which generally is where the kerblam.toml
/// file is.
///
/// This causes each project to have its own cache file.
pub fn get_cache_path() -> Result<PathBuf> {
    let cache_dir = match get_my_home() {
        Ok(dir) => dir
            .unwrap_or_else(|| PathBuf::from_str("/tmp").unwrap())
            .join(".cache/kerblam"),
        Err(_) => PathBuf::from_str("/tmp/.cache/kerblam").unwrap(),
    };
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    };
    let path_hash = calc_hash(&current_dir().unwrap());
    let cache_path = cache_dir.join(format!("{}", path_hash));
    log::debug!("Determined cache path: {:?}", cache_path);

    Ok(cache_path)
}

/// Read the content of the cache file to a string
/// Returns None if a cache cannot be found.
pub fn get_cache() -> Option<Cache> {
    let cache_file = get_cache_path().unwrap();
    if !cache_file.exists() {
        return None;
    }
    let conn = match File::open(cache_file.clone()) {
        Ok(file) => file,
        Err(_) => {
            log::warn!("Cache file cannot be read. Returning 'None' for cache content.");
            return None;
        }
    };

    let cache_content = match serde_json::from_reader(conn) {
        Ok(content) => content,
        Err(e) => {
            log::error!(
                "Failed to parse cache content: {e:?} Returning None. Consider deleting {cache_file:?}."
            );
            return None;
        }
    };

    Some(cache_content)
}

/// Save content to the cache
pub fn write_cache(content: Cache) -> Result<()> {
    let cache_file = get_cache_path().unwrap();
    std::fs::write(cache_file, serde_json::to_string_pretty(&content)?)?;

    Ok(())
}

/// Check the name of the last profile run in this project and return
/// True if it's the same as the current one, False otherwise.
/// Returns None if there is no cache.
///
/// Also updates the cache to the new profile
#[allow(clippy::needless_update)]
pub fn check_last_profile(current_profile: String) -> Option<bool> {
    let last_profile = get_cache();

    let result = last_profile.clone().is_some_and(|x| {
        x.last_executed_profile
            .is_some_and(|x| x == current_profile)
    });

    let new_cache = Cache {
        last_executed_profile: Some(current_profile),
        // This line is where clippy complains: there is no need to do this
        // as Cache has only the `last_executed_profile` field.
        // But - just for now. It's added for future compatibility
        ..last_profile.clone().unwrap_or_default()
    };
    match write_cache(new_cache) {
        Ok(_) => {}
        Err(_) => {
            log::error!("New cache was generated but could not be written. Silently continuing.")
        }
    };

    if last_profile.is_none() {
        None
    } else {
        Some(result)
    }
}

/// Delete from the cache the last profile used.
///
/// This currently just deletes the cache.
#[allow(clippy::needless_update)]
pub fn delete_last_profile() -> Result<()> {
    let cache = get_cache();

    let new_cache = Cache {
        last_executed_profile: None,
        // This line is where clippy complains: there is no need to do this
        // as Cache has only the `last_executed_profile` field.
        // But - just for now. It's added for future compatibility
        ..cache.unwrap_or_default()
    };

    write_cache(new_cache)?;

    Ok(())
}

/// Calculate the hash of some hashable object
fn calc_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
