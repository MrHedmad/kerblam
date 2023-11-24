use std::error::Error;
use std::fs;
use std::path::{PathBuf, Path};

use indicatif::ProgressIterator;
use tempfile::{TempDir, tempdir};
use log;

use crate::commands::new::normalize_path;
use crate::errors::StopError;
use crate::options::KerblamTomlOptions;

enum ExecutionStrategy {
    DockerizedMake,
    DockerizedShell,
    Make,
    Shell,
}

fn create_execution_directory(
    original_dir: impl AsRef<Path>,
    data_dir: impl AsRef<Path>,
    data_files: impl AsRef<Path>
    ) -> Result<TempDir, StopError> {
    let exec_dir = tempdir().unwrap();
    log::info!("Created temporary dir: {:?}", exec_dir);
    let data_dir: &Path = data_dir.as_ref();

    // Copy everything in the original folder except for the data_dir
    let copy_files: Vec<PathBuf> = fs::read_dir(original_dir).unwrap()
        .into_iter()
        .filter_map(|x| x.ok())
        .filter_map(|path| if path.path() != data_dir {Some(path.path())} else {None})
        .filter(|path| path.exists()) // This is redundant but just to be safe
        .collect();

    
    for path in copy_files.iter().progress() {
        fs::copy(
            path,
            exec_dir.path()
            .join(
                path.strip_prefix(original_dir)
                .unwrap()) // I think unwrap here is safe.
            );
    }
    
    // Create all of the data symlinks
    // The FileLinks refer to the standard directory. We need to convert
    // them to the new, temporary directory.
    
    todo!()
}

fn create_file_links(data_dir: impl AsRef<Path>, renames: Vec<FileLink>) -> Result<Vec<FileLink>, StopError> {
    unimplemented!()
}

struct FileLink {
    from: PathBuf,
    to: PathBuf
}

impl FileLink {
    fn link(&self) -> Result<(), StopError> {
        unimplemented!()
    }
}

fn extract_profile_paths(
    config: KerblamTomlOptions,
    profile_name: &str,
) -> Result<Vec<FileLink>, Box<dyn Error>> {
    // The call here is broken because the last `ok_or` makes a temporary
    // reference that does not live enough until the 'profile.iter()' call
    // later on. I'm not sure why this is the case, but separating the
    // calls fixes it.
    let profiles = config
        .data
        .ok_or("Missing 'data' field!")?
        .profiles
        .ok_or("Missing 'profiles' field!")?;

    let profile = profiles
        .get(profile_name)
        .ok_or(format!("Could not find {} profile", profile_name))?;

    Ok(profile
       .iter()
       .map(|(from, to)| FileLink {from, to})
       .collect()
    )
}

pub fn kerblam_run_project(
    config: KerblamTomlOptions,
    module_name: String,
    runtime_dir: &PathBuf,
    profile: Option<String>,
) -> Result<String, StopError> {
    // Check if the paths that we need are present
    let mut expected_paths: Vec<PathBuf> = vec!["./data/in", "./src/pipes", "./src/dockerfiles"]
        .into_iter()
        .map(|x| runtime_dir.join(x))
        .collect();
    let module_file = &mut runtime_dir.join("./src/pipes");
    module_file.push(module_name.trim().trim_matches('/'));
    expected_paths.push(module_file.to_owned());

    if expected_paths
        .into_iter()
        .map(|path| {
            if !path.exists() {
                println!("🔥 Could not find {:?}", normalize_path(path.as_path()));
                true // i.e "Please crash"
            } else {
                false // i.e "Nothing to see here"
            }
        })
        .any(|x| x)
    {
        return Err(StopError {
            msg: "Could not find required files.".to_string(),
        });
    };

    // Handle renaming the input files if we are in a profile
    let unwinding_paths: Vec<FileLink> = if let Some(profile) = profile {
        // This should mean that there is a profile with the same name in the
        // config...
        let profile_paths = match extract_profile_paths(config, profile.as_str()) {
            Ok(p) => p,
            Err(e) => return Err(StopError { msg: e.to_string() }),
        };

        // Check that we can find the files that we need to rename.
        if profile_paths
            .iter()
            .map(|x| { match x.check() {
                Ok(_) => false,
                Err(_) => {
                    println!("🔥 Could not find {}!", x.from.to_string_lossy());
                    true
                }
            }})
            .any(|x| x)
        {
            return Err(StopError { msg: "Could not find profile files.".to_string() });
        };
        
        // Rename the paths
        for path in profile_paths.iter() {
            match path.execute() {
                Ok(_) => println!("")
            };
        }

        profile_paths.into_iter().map(|x| x.invert()).collect()
    } else {
        vec![]
    };

    // Move the files that we need to move

    // Execute the run

    // Undo the input file renaming

    // Return either an error or OK, if the pipeline finished appropriately
    // or crashed and burned.

    Ok(String::from("Done!"))
}
