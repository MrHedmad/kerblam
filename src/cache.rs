use std::env::current_dir;
use std::fs::read_to_string;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use homedir::get_my_home;

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
    Ok(cache_dir.join(format!("{}", path_hash)))
}

/// Read the content of the cache file to a string
/// Returns None if a cache cannot be found.
pub fn get_cache() -> Option<String> {
    let cache_file = get_cache_path().unwrap();
    if !cache_file.exists() {
        return None;
    }

    Some(read_to_string(cache_file).unwrap())
}

/// Save content to the cache
pub fn write_cache(name: &str) -> Result<()> {
    let cache_file = get_cache_path().unwrap();
    std::fs::write(cache_file, name)?;

    Ok(())
}

/// Check the name of the last profile run in this project and return
/// True if it's the same as the current one, False otherwise.
/// Returns None if there is no cache.
///
/// Also updates the cache to the new profile
pub fn check_last_profile(current_profile: String) -> Option<bool> {
    let last_profile = get_cache();

    let result = last_profile
        .clone()
        .is_some_and(|x| x.trim() == current_profile);

    write_cache(&current_profile).unwrap();

    if last_profile.is_none() {
        None
    } else {
        Some(result)
    }
}

/// Delete from the cache the last profile used.
///
/// This currently just deletes the cache.
pub fn delete_last_profile() -> Result<()> {
    let cache_path = get_cache_path()?;

    if cache_path.exists() {
        std::fs::remove_file(cache_path)?;
    }

    Ok(())
}

fn calc_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
