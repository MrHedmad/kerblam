use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_to_string;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::execution::{setup_ctrlc_hook, Executor, FileMover};
use crate::options::KerblamTomlOptions;
use crate::options::Pipe;

use anyhow::{anyhow, bail, Result};
use filetime::{set_file_mtime, FileTime};
use homedir::get_my_home;

/// Push a bit of a string to the end of this path
///
/// Useful if you want to add an extension to the path.
/// Requires a clone.
fn push_fragment(buffer: impl AsRef<Path>, ext: &str) -> PathBuf {
    let buffer = buffer.as_ref();
    let mut path = buffer.as_os_str().to_owned();
    path.push(ext);
    path.into()
}

fn infer_test_data(paths: Vec<PathBuf>) -> HashMap<PathBuf, PathBuf> {
    let mut matches: HashMap<PathBuf, PathBuf> = HashMap::new();

    for path in paths.clone() {
        let file_name = path.file_name().unwrap().to_string_lossy();
        if file_name.starts_with("test_") {
            let slug = file_name.trim_start_matches("test_");
            let potential_target = path.clone().with_file_name(slug);
            if paths.iter().any(|x| *x == potential_target) {
                matches.insert(potential_target, path);
            }
        }
    }

    matches
}

/// NOTE: This chache-related stuff may be best moved to its own module and
/// perhaps wrapped in a struct. The functions are broken up like this to be
/// a bit more flexible if we need to add more than a single word in the cache
/// in the future (like, using JSON or stuff like that).
/// For now, it's so simple that just a few loose functions should be enough.

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

fn extract_profile_paths(
    config: &KerblamTomlOptions,
    profile_name: &str,
) -> Result<Vec<FileMover>> {
    let root_dir = config.input_data_dir();
    // The call here was broken in 2 because the last `ok_or` makes a temporary
    // reference that does not live enough until the 'profile.iter()' call
    // later on. I'm not sure why this is the case, but separating the
    // calls fixes it.
    let mut profiles = config
        .clone()
        .data
        .ok_or(anyhow!("Missing 'data' field!"))?
        .profiles
        .ok_or(anyhow!("Missing 'profiles' field!"))?;

    // add the default 'test' profile
    if !profiles.keys().any(|x| x == "test") {
        let input_files = config.input_files();
        let inferred_test = infer_test_data(input_files);
        if !inferred_test.is_empty() {
            log::debug!("Inserted inferred test profile: {inferred_test:?}");
            profiles.insert("test".to_string(), inferred_test);
        }
    }

    let profile = profiles
        .get(profile_name)
        .ok_or(anyhow!("Could not find {} profile", profile_name))?;

    // Check if the sources exist, otherwise we crash now, and not later
    // when we actually move the files.
    let exist_check: Vec<anyhow::Error> = profile
        .iter()
        .flat_map(|(a, b)| [a, b])
        .map(|file| {
            let f = &root_dir.join(file);
            log::debug!("Checking if {f:?} exists...");
            if !f.exists() {
                bail!("\t- {:?} does not exists!", file)
            };
            Ok(())
        })
        .filter_map(|x| x.err())
        .collect();

    if !exist_check.is_empty() {
        let mut missing: Vec<String> = Vec::with_capacity(exist_check.len());
        for item in exist_check {
            missing.push(item.to_string());
        }
        bail!(
            "Failed to find some profiles files:\n{}",
            missing.join("\n")
        )
    }

    // Also check if the targets do NOT exist, so we don't overwrite anything
    let exist_check: Vec<anyhow::Error> = profile
        .iter()
        .flat_map(|(a, b)| [a, b])
        .map(|file| {
            let f = &root_dir.join(push_fragment(file, ".original"));
            log::debug!("Checking if {f:?} destroys files...");
            if f.exists() {
                bail!("\t- {:?} would be destroyed by {:?}!", f, file)
            };
            Ok(())
        })
        .filter_map(|x| x.err())
        .collect();

    if !exist_check.is_empty() {
        let mut missing: Vec<String> = Vec::with_capacity(exist_check.len());
        for item in exist_check {
            missing.push(item.to_string());
        }
        bail!(
            "Some profile temporary files would overwrite real files:\n{}",
            missing.join("\n")
        )
    }

    Ok(profile
        .iter()
        .flat_map(|(original, profile)| {
            // We need two FileMovers. One for the temporary file
            // that holds the original file (e.g. 'to'), and one for the
            // profile-to-original rename.
            // To unwind, we just redo the transaction, but in reverse.
            [
                // This one moves the original to the temporary file
                FileMover::from((
                    &root_dir.join(original),
                    &root_dir.join(push_fragment(original, ".original")),
                )),
                // This one moves the profile one to the original one
                FileMover::from((&root_dir.join(profile), &root_dir.join(original))),
            ]
        })
        .collect())
}

pub fn kerblam_run_project(
    config: KerblamTomlOptions,
    pipe: Pipe,
    runtime_dir: &PathBuf,
    profile: Option<String>,
    ignore_container: bool,
    extra_args: Option<Vec<String>>,
) -> Result<String> {
    let pipe = if ignore_container {
        pipe.drop_env()
    } else {
        pipe
    };

    // Create an executor for later.
    let executor: Executor = pipe.into_executor(runtime_dir)?;

    // From here on we should not crash. Therefore, we have to catch SIGINTs
    // as the come in.
    let sigint_rec = setup_ctrlc_hook().expect("Failed to setup SIGINT hook!");

    // Handle renaming the input files if we are in a profile
    let unwinding_paths: Vec<FileMover> = if let Some(profile) = profile.clone() {
        // This should mean that there is a profile with the same name in the
        // config...
        let profile_paths = extract_profile_paths(&config, profile.as_str())?;

        // Check the cache (if there) what the last profile was.
        // If it was this one, we should not update the file creation time
        // as we move them around, or the make pipelines re-run from the
        // beginning even if we did nothing to them
        let cached_run = check_last_profile(profile);
        let cached_run = cached_run.unwrap_or(false);
        log::debug!("Checked cached profile: {}", cached_run);

        // Rename the paths that we found
        let move_results: Vec<Result<FileMover, anyhow::Error>> = profile_paths
            .into_iter()
            .map(|x| x.rename(!cached_run))
            .collect();
        // If they are all ok, return the vec
        if move_results.iter().all(|x| x.is_ok()) {
            move_results.into_iter().map(|x| x.unwrap()).collect()
        } else {
            // Not all is ok, unwind and bail out.
            // I have to clone here as I need to consume the vector twice,
            // but the toplevel vector cannot be cloned, so I clone and then
            // ref deeper in. A bit clunky.
            let unwindable: Vec<FileMover> = move_results
                .iter()
                .filter_map(|x| x.as_ref().ok())
                .map(|x| x.to_owned())
                .collect();
            for mover in unwindable {
                // I don't use the result for the same reason.
                let _ = mover.rename(!cached_run);
            }

            let failed: Vec<anyhow::Error> =
                move_results.into_iter().filter_map(|x| x.err()).collect();

            bail!(
                "Some profiled paths failed to be moved: {}",
                failed
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
    } else {
        // If we are not in a profile now, but we were before, we should
        // re-touch all the old profile paths just to be safe that the
        // whole pipeline is re-run again with the new data
        let last_profile = get_cache();
        if last_profile.is_some() {
            log::debug!("Should re-touch profile files.");
            let profile_paths = extract_profile_paths(&config, &last_profile.unwrap())?;

            for mover in profile_paths {
                log::debug!("Touching {:?}", &mover.clone().get_from());
                set_file_mtime(&mover.get_from(), FileTime::now())?;
            }

            // We are done. We can delete the last profile.
            let _ = delete_last_profile();
        }

        vec![]
    };

    // Build the extra env vars that we want to set during the execution
    let env_vars: HashMap<String, String> = if let Some(profile) = profile {
        HashMap::from([("KERBLAM_PROFILE".to_string(), profile)])
    } else {
        HashMap::new()
    };

    // Execute the executor
    let runtime_result = executor.execute(sigint_rec, &config, env_vars, extra_args);

    // Undo the input file renaming
    if !unwinding_paths.is_empty() {
        log::info!("Undoing profile...");
        for item in unwinding_paths.into_iter().rev() {
            // If this worked before, it should work now, that is why I discard the
            // result...
            // TODO: This might be a bad idea.
            //
            // We can skip updating timestamps at this stage
            let _ = item.rename(false);
        }
    }

    // Try and destroy the symlinks

    // Return either an error or OK, if the pipeline finished appropriately
    // or crashed and burned.
    if runtime_result.is_ok() {
        match runtime_result.unwrap() {
            Some(res) => {
                if res.success() {
                    Ok("Done!".into())
                } else {
                    Err(anyhow!("Process exited with error: {res:?}"))
                }
            }
            None => Err(anyhow!("Process killed.")),
        }
    } else {
        Err(anyhow!("Process exited."))
    }
}
