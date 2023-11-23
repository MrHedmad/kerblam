use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::commands::new::normalize_path;
use crate::errors::StopError;
use crate::options::KerblamTomlOptions;

enum ExecutionStrategy {
    DockerizedMake,
    DockerizedShell,
    Make,
    Shell,
}

struct PathRenamer {
    from: PathBuf,
    to: PathBuf,
}

impl PathRenamer {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        fs::rename(&self.from, &self.to)?;

        Ok(())
    }

    // If I get this, `self` means that we have to destroy the original
    // PathRenamer, which is what we want.
    fn invert(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }
}

// If you want to be a bit more generic, then you can do this:
// Or, replace the Into<PathBuf> with AsRef<Path>, then the from/to will look
// like value.X.as_ref().into()
impl<F: Into<PathBuf>, T: Into<PathBuf>> From<(F, T)> for PathRenamer {
    fn from(value: (F, T)) -> Self {
        Self {
            from: value.0.into(),
            to: value.1.into(),
        }
    }
}

fn extract_profile_paths(
    config: KerblamTomlOptions,
    profile_name: &str,
) -> Result<Vec<PathRenamer>, Box<dyn Error>> {
    let profile = config
        .data
        .ok_or("Missing 'data' field!")?
        .profiles
        .ok_or("Missing 'profiles' field!")?
        .get(profile_name)
        .ok_or(format!("Could not find {} profile", profile_name))?;

    Ok(profile
        .iter()
        .map(|(from, to)| PathRenamer::from((from, to)))
        .collect())
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
    let unwinding_paths: Vec<PathRenamer> = if let Some(profile) = profile {
        // This should mean that there is a profile with the same name in the
        // config...
        let profile_paths = match extract_profile_paths(config, profile.as_str()) {
            Ok(p) => p,
            Err(e) => return Err(StopError { msg: e.to_string() }),
        };

        // Rename the paths that we found

        if profile_paths
            .iter()
            .map(|x| match x.execute() {
                Ok(_) => false,
                Err(msg) => {
                    println!("🔥 Could not find {}!", msg);
                    true
                }
            })
            .any(|x| x)
        {
            return Err(StopError {
                msg: "Could not find profile files.".to_string(),
            });
        };

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
