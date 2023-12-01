use log;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};

use crate::commands::new::normalize_path;
use crate::options::KerblamTomlOptions;

use anyhow::{anyhow, bail, Result};
use crossbeam_channel::{bounded, Receiver};
use ctrlc;

#[allow(dead_code)]
enum CommandResult {
    Exited { res: ExitStatus },
    Killed,
}

fn setup_ctrlc_hook() -> Result<Receiver<bool>> {
    let (sender, receiver) = bounded(2);

    let multiple_guard = receiver.clone();

    ctrlc::try_set_handler(move || {
        let _ = sender.send(true);
        if multiple_guard.is_full() {
            panic!("Got two CTRL-C without a consumer.")
        }
    })?;

    Ok(receiver)
}

fn run_protected_command<F>(cmd_builder: F, receiver: Receiver<bool>) -> Result<CommandResult>
where
    F: FnOnce() -> Child,
{
    // check if we have to kill the child even before being spawned
    if let Ok(true) = receiver.try_recv() {
        bail!("Not starting with pending SIGINT!")
    };

    let mut child = cmd_builder();

    loop {
        // Check every 50 ms how the child is faring.
        std::thread::sleep(std::time::Duration::from_millis(50));

        // If we got a kill signal, kill the child, obi-wan kenobi!
        if let Ok(true) = receiver.try_recv() {
            match child.kill() {
                Ok(_) => return Ok(CommandResult::Killed),
                Err(_) => {
                    bail!("Failed to kill child!")
                }
            }
        };

        // Check if the children is done
        if let Some(status) = child.try_wait().expect("Where did the child go?") {
            return Ok(CommandResult::Exited { res: status });
        };
    }
}

struct Executor {
    target: FileMover,
    env: Option<FileMover>,
    strategy: ExecutionStrategy,
    // executor_path
}

// TODO: I think we can add all cleanup code to `Drop`, so that a lot of these
// functions can be simplified a lot.
// E.g. make a "cleanup: Option<Vec<PathBuf>>" to the Executor and remove
// (without returing the fail) the files specified in the vector (if any)
// so that we can stop at any time and still be sure to cleanup.
// The same idea could be used for the FileRenamers, but we'd need to be
// careful on when they are dropped.

impl Executor {
    /// Execute this executor based on its data
    ///
    /// Either builds and runs a docker container, or executes make and/or
    /// bash, depending on the strategy.
    ///
    /// Destroys itself in the process.
    fn execute(self, signal_receiver: Receiver<bool>) -> Result<()> {
        let mut cleanup: Vec<PathBuf> = vec![];
        cleanup.push(self.target.copy()?);
        if let Some(target) = self.env.clone() {
            cleanup.push(target.copy()?);
        }

        let command_args = if self.env.is_some() {
            // This is a dockerized run.
            let builder = || {
                Command::new("docker")
                    // If the `self.env` path is not UTF-8 I'll eat my hat.
                    .args(["build", "--tag", "kerblam_runtime", "."])
                    .stdout(Stdio::inherit())
                    .stdin(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .expect("Failed to spawn builder process.")
            };

            let success = match run_protected_command(builder, signal_receiver.clone()) {
                Ok(CommandResult::Exited { res: _ }) => true,
                Ok(CommandResult::Killed) => false,
                Err(_) => false,
            };

            if !success {
                // Cleanup early.
                log::debug!("Failed to build docker environment. Unwinding early.");

                for file in cleanup {
                    // The idea is that this cleanup should not fail, and anyway
                    // we don't really care if it does or not.
                    let _ = fs::remove_file(file);
                }

                bail!("Command exited with an error.",);
            };

            vec![
                "docker",
                "run",
                "-it",
                "-v",
                "./data:/data",
                "kerblam_runtime",
            ]
        } else {
            // This is a normal run.
            match self.strategy {
                ExecutionStrategy::Make => {
                    vec!["make", "-f", self.target.to.to_str().unwrap()]
                }
                ExecutionStrategy::Shell => {
                    vec!["bash", self.target.to.to_str().unwrap()]
                }
            }
        };

        let mut command = Command::new(command_args[0]);

        let builder = || {
            command
                .args(&command_args[1..command_args.len()])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .spawn()
                .expect("Cannot retrieve command output!")
        };

        match run_protected_command(builder, signal_receiver) {
            Ok(CommandResult::Exited { res: _ }) => (), // We don't care if it succeeded.
            Ok(CommandResult::Killed) => {
                eprintln!("\nChild process exited early. Continuing to cleanup...")
            }
            Err(e) => eprintln!("\nChild process failure: {}\nContinuing to cleanup...", e),
        };

        for file in cleanup {
            // The idea is that this cleanup should not fail, and anyway
            // we don't really care if it does or not.
            let _ = fs::remove_file(file);
        }

        Ok(())
    }

    fn create(project_path: impl AsRef<Path>, module_name: &str) -> Result<Self> {
        let project_path = project_path.as_ref();
        let makefile = project_path.join("src/pipes/".to_string() + module_name + ".makefile");
        let shellfile = project_path.join("src/pipes/".to_string() + module_name + ".sh");
        let dockerfile =
            project_path.join("src/dockerfiles/".to_string() + module_name + ".dockerfile");

        let target: PathBuf;
        let strategy = if makefile.exists() & shellfile.exists() {
            bail!(
                "Found both a makefile and a shellfile for module {}. Aborting!",
                module_name
            );
        } else if makefile.exists() {
            target = makefile;
            ExecutionStrategy::Make
        } else if shellfile.exists() {
            target = shellfile;
            ExecutionStrategy::Shell
        } else {
            bail!(
                "Could not find either a shellfile or a makefile named {}!",
                module_name
            );
        };

        if dockerfile.exists() {
            Ok(Self {
                target: FileMover {
                    from: target,
                    to: project_path.join("executor"),
                },
                env: Some(FileMover {
                    from: dockerfile,
                    to: project_path.join("Dockerfile"),
                }),
                strategy,
            })
        } else {
            Ok(Self {
                target: FileMover {
                    from: target,
                    to: project_path.join("executor"),
                },
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
        log::debug!("Moving {:?} to {:?}", self.from, self.to);
        fs::rename(&self.from, &self.to)?;

        Ok(self.invert())
    }

    /// Symlink the two paths, without any actual move.
    ///
    /// Returns the location of the created symlink.
    fn _link(&self) -> Result<PathBuf> {
        // TODO: Make this compatible with other operating systems.
        log::debug!("Linking {:?} to {:?}", self.from, self.to);
        std::os::unix::fs::symlink(self.from.clone(), self.to.clone())?;

        Ok(self.to.clone())
    }

    /// Copy from to to.
    ///
    /// Returns the location of the created file.
    fn copy(&self) -> Result<PathBuf> {
        log::debug!("Copying {:?} to {:?}", self.from, self.to);
        fs::copy(self.from.clone(), self.to.clone())?;

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
    fn from(value: (F, T)) -> Self {
        Self {
            from: value.0.into(),
            to: value.1.into(),
        }
    }
}

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

fn extract_profile_paths(
    config: KerblamTomlOptions,
    profile_name: &str,
    root_dir: impl AsRef<Path>,
) -> Result<Vec<FileMover>> {
    let root_dir = root_dir.as_ref();
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

    // Check if the sources exist, otherwise we crash now, and not leter
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

    // From here on we should not crash. Therefore, we have to catch SIGINTs
    // as the come in.
    let sigint_rec = setup_ctrlc_hook().expect("Failed to setup SIGINT hook!");

    // Handle renaming the input files if we are in a profile
    let unwinding_paths: Vec<FileMover> = if let Some(profile) = profile {
        // This should mean that there is a profile with the same name in the
        // config...
        let profile_paths =
            extract_profile_paths(config, profile.as_str(), runtime_dir.join("./data/in/"))?;
        // Rename the paths that we found
        let move_results: Vec<Result<FileMover, anyhow::Error>> =
            profile_paths.into_iter().map(|x| x.rename()).collect();
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
                let _ = mover.rename();
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
        vec![]
    };

    // Execute the executor
    let runtime_result = executor.execute(sigint_rec);

    // Undo the input file renaming
    if !unwinding_paths.is_empty() {
        log::info!("Undoing profile...");
        for item in unwinding_paths.into_iter().rev() {
            // If this worked before, it should work now, that is why I discard the
            // result...
            // TODO: This might be a bad idea.
            let _ = item.rename();
        }
    }

    // Try and destroy the symlinks

    // Return either an error or OK, if the pipeline finished appropriately
    // or crashed and burned.
    if runtime_result.is_ok() {
        Ok(String::from("Done!"))
    } else {
        Err(anyhow!("Process exited."))
    }
}
