use std::ffi::OsStr;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::{Command, Stdio, Output};

use crate::commands::new::normalize_path;
use crate::options::KerblamTomlOptions;

use anyhow::{anyhow, bail, Result};

struct Executor {
    target: FileMover,
    env: Option<FileMover>,
    strategy: ExecutionStrategy,
    // executor_path
}

impl Executor {
    /// Execute this executor based on its data
    ///
    /// Either builds and runs a docker container, or executes make and/or
    /// bash, depending on the strategy.
    ///
    /// Destroys itself in the process.
    fn execute(self: Self) -> Result<Output> {
        let mut cleanup: Vec<PathBuf> = vec![];
        cleanup.push(self.target.link()?);
        if let Some(target) = self.env.clone() {
            cleanup.push(target.link()?);
        }


        let command_args = if self.env.is_some() {
            // This is a dockerized run.
            let builder = Command::new("docker")
                // If the `self.env` path is not UTF-8 I'll eat my hat.
                .args(["build", "-f", self.env.unwrap().to.to_str().unwrap(), "--tag", "kerblam_runtime"])
                .stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .output()
                .expect("Failed to spawn builder process.");
            
            if ! builder.status.success() {
                bail!("Cannot build execution environment!")
            };

            vec!["docker", "run", "-it", "-v", "./data:/data", "kerblam_runtime"]

        } else {
            // This is a normal run.
            match self.strategy {
                ExecutionStrategy::Make => {
                    vec!["make", "-f", self.target.to.to_str().unwrap()]
                },
                ExecutionStrategy::Shell => {
                    vec!["bash", self.target.to.to_str().unwrap()]
                }
            }
        };

        let mut command = Command::new(command_args[0]);

        let result = command
            .args(&command_args[1..command_args.len()])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Cannot retrieve command output!");
        
        for file in cleanup {
            // The idea is that this cleanup should not fail, and anyway
            // we don't really care if it does or not.
            let _ = fs::remove_file(file);
        };

        Ok(result)

    }

    fn create(project_path: impl AsRef<Path>, module_name: &str) -> Result<Self> {
        let project_path = project_path.as_ref();
        let makefile = project_path.join("src/pipes/".to_string() + module_name + ".makefile");
        let shellfile = project_path.join("src/pipes/".to_string() + module_name + ".sh");
        let dockerfile = project_path.join("src/dockerfiles/".to_string() + module_name + ".dockerfile");
        
        let target: PathBuf;
        let strategy = if makefile.exists() & shellfile.exists() {
            bail!("Found both a makefile and a shellfile for module {}. Aborting!", module_name);
        } else if makefile.exists() {
            target = makefile;
            ExecutionStrategy::Make
        } else if shellfile.exists() {
            target = shellfile;
            ExecutionStrategy::Shell
        } else {
            bail!("Could not find either a shellfile or a makefile named {}!", module_name);
        };

        if dockerfile.exists() { 
            Ok(Self {
                target: FileMover { from: target, to: project_path.join("executor") },
                env: Some(FileMover{ from: dockerfile, to: project_path.join("Dockerfile")} ),
                strategy
            })
        } else {
            Ok(Self {
                target: FileMover { from: target, to: project_path.join("executor") },
                env: None,
                strategy,
            })
        }

    }
}

enum ExecutionStrategy {
    Make,
    Shell,
}

#[derive(Debug, Clone)]
struct FileMover {
    from: PathBuf,
    to: PathBuf,
}

impl FileMover {
    /// Rename the files, destroying this file mover and making the reverse.
    fn rename(self) -> Result<FileMover> {
        fs::rename(&self.from, &self.to)?;

        Ok(self.invert())
    }

    /// Symlink the two paths, without any actual move.
    ///
    /// Returns the location of the created symlink.
    fn link(&self) -> Result<PathBuf> {
        // TODO: Make this compatible with other operating systems.
        std::os::unix::fs::symlink(self.from.clone(), self.to.clone())?;

        Ok(self.to.clone())
    }

    /// Invert this object, creating a new one with swapped from/to fields.
    ///
    /// Takes ownership of itself, therefore destroying the original.
    fn invert(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }
}

impl<F: Into<PathBuf>, T: Into<PathBuf>> From<(F, T)> for FileMover {
    fn from (value: (F, T)) -> Self {
        Self {
            from: value.0.into(),
            to: value.1.into(),
        }
    }
}


fn extract_profile_paths(
    config: KerblamTomlOptions,
    profile_name: &str,
) -> Result<Vec<FileMover>> {
    // The call here was broken in 2 because the last `ok_or` makes a temporary
    // reference that does not live enough until the 'profile.iter()' call
    // later on. I'm not sure why this is the case, but separating the
    // calls fixes it.
    let profiles = config
        .data
        .ok_or(anyhow!("Missing 'data' field!"))?
        .profiles
        .ok_or(anyhow!("Missing 'profiles' field!"))?;

    let profile = profiles
        .get(profile_name)
        .ok_or(anyhow!("Could not find {} profile", profile_name))?;

    Ok(profile
       .iter()
       .flat_map(|(from, to)| {
           // We need two FileMovers. One for the temporary file
           // that holds the original file (e.g. 'to'), and one for the
           // profile-to-original rename.
           // To unwind, we just redo the transaction, but in reverse.
           let new_extension = to.extension().unwrap_or(&OsStr::new(""));
           new_extension.to_os_string().push("original");

           [
               FileMover::from((to, to.with_extension(new_extension))),
               FileMover::from((from, to))
           ]

       })
       .collect()
    )
}

pub fn kerblam_run_project(
    config: KerblamTomlOptions,
    module_name: String,
    runtime_dir: &PathBuf,
    profile: Option<String>,
) -> Result<String> {
    // Check if the paths that we need are present
    let expected_paths: Vec<PathBuf> = vec!["./data/in", "./src/pipes", "./src/dockerfiles"]
        .into_iter()
        .map(|x| runtime_dir.join(x))
        .collect();
    let module_file = &mut runtime_dir.join("./src/pipes");
    module_file.push(module_name.trim().trim_matches('/'));

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
        bail!("Could not find required files.")
    };

    // Create an executor for later.
    let executor: Executor = Executor::create(runtime_dir, module_name.as_str())?;

    // Handle renaming the input files if we are in a profile
    
    let unwinding_paths: Vec<FileMover> = if let Some(profile) = profile {
        // This should mean that there is a profile with the same name in the
        // config...
        let profile_paths = extract_profile_paths(config, profile.as_str())?;
        // Rename the paths that we found
        let move_results: Vec<Result<FileMover, anyhow::Error>> = profile_paths
            .into_iter()
            .map(|x| x.rename())
            .collect();
        // If they are all ok, return the vec
        if move_results.iter().all(|x| x.is_ok()) {
            move_results.into_iter().map(|x| x.unwrap()).collect()
        } else {
            // Not all is ok, unwind and bail out.
            // I have to clone here as I need to consume the vector twice,
            // but the toplevel vector cannot be cloned, so I clone and then
            // ref deeper in. A bit clunky.
            let unwindable: Vec<FileMover> = move_results.iter().filter_map(|x| x.clone().as_ref().ok()).map(|x| x.to_owned()).collect();
            for mover in unwindable {
                // I don't use the result for the same reason.
                let _ = mover.rename();
            }

            let failed: Vec<anyhow::Error> = move_results.into_iter().filter_map(|x| x.err()).collect();

            bail!("Some profiled paths failed to be moved: {}", failed.into_iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"))
        }
    } else {
        vec![]
    };
    
    // Execute the executor
    let runtime_result = executor.execute()?;

    // Undo the input file renaming
    if unwinding_paths.len() != 0 {
        log::info!("Undoing profile...");
        for item in unwinding_paths.into_iter().rev() {
            // If this worked before, it should work now, that is why I discard the
            // result...
            // TODO: This might be a bad idea.
            let _ = item.rename();
        }
    }

    // Return either an error or OK, if the pipeline finished appropriately
    // or crashed and burned.
    if runtime_result.status.success() {
        Ok(String::from("Done!"))
    } else {
        Err(anyhow!("Process exited with exit code {:?}", runtime_result.status.code()))
    }
}

