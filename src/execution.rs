use core::panic;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};

use crate::options::KerblamTomlOptions;
use crate::utils::normalize_path;

use anyhow::{anyhow, bail, Context, Result};
use crossbeam_channel::{bounded, Receiver};
use ctrlc;
use filetime::{set_file_mtime, FileTime};

pub fn setup_ctrlc_hook() -> Result<Receiver<bool>> {
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

pub struct Executor {
    target: FileMover,
    env: Option<PathBuf>,
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

macro_rules! stringify {
    ( $x:expr ) => {{
        $x.into_iter().map(|x| x.to_string()).collect()
    }};
}

/// Generate a series of strings suitable for -v options to bind the data dirs
fn generate_bind_mount_strings(config: &KerblamTomlOptions) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    let root = current_dir().unwrap();

    let dirs = vec![
        config.input_data_dir(),
        config.output_data_dir(),
        config.intermediate_data_dir(),
    ];

    let host_workdir = config.execution.workdir.clone();

    for dir in dirs {
        // the folder here, in the local file system
        let local = dir.to_string_lossy().to_string();
        // the folder in the host container
        let host = dir.strip_prefix(root.clone()).unwrap().to_string_lossy();
        let host = host_workdir.join(format!("./{}", host));
        let host = normalize_path(host.as_ref());
        let host = host.to_string_lossy();

        result.push(format!("{}:{}", local, host))
    }

    result
}

impl Executor {
    /// Execute this executor based on its data
    ///
    /// Either builds and runs a container, or executes make and/or
    /// bash, depending on the strategy.
    ///
    /// Needs the kerblam config to work, as we need to bind-mount the
    /// same data paths as locally needed.
    ///
    /// Destroys itself in the process.
    pub fn execute(
        self,
        signal_receiver: Receiver<bool>,
        config: &KerblamTomlOptions,
        env_vars: HashMap<String, String>,
    ) -> Result<()> {
        let mut cleanup: Vec<PathBuf> = vec![];

        let command_args = if self.env.is_some() {
            // This is a containerized run
            let backend: String = config.execution.backend.clone().into();
            let runtime_name = self.build_env(signal_receiver.clone(), &backend)?;
            let mut partial: Vec<String> = stringify![vec![&backend, "run", "-it"]];
            // We need to bind-mount the same data dirs as specified in the options
            let mounts = generate_bind_mount_strings(config);
            for mount in mounts {
                partial.extend(vec!["-v".to_string(), mount].into_iter())
            }

            // Add the correct entrypoint override
            let workdir = config.execution.workdir.clone();
            let workdir = workdir.to_string_lossy();
            let execution_command: Vec<String> = match self.strategy {
                ExecutionStrategy::Make => stringify!(vec![
                    "--entrypoint",
                    "make",
                    &runtime_name,
                    "-f",
                    &format!("{}/executor", workdir)
                ]),
                ExecutionStrategy::Shell => stringify!(vec![
                    "--entrypoint",
                    "bash",
                    &runtime_name,
                    &format!("{}/executor", workdir)
                ]),
            };

            partial.extend(execution_command);

            partial
        } else {
            // This is a normal run.
            // Move the executor file
            cleanup.push(self.target.copy()?);

            match self.strategy {
                ExecutionStrategy::Make => {
                    stringify![vec!["make", "-f", self.target.to.to_str().unwrap()]]
                }
                ExecutionStrategy::Shell => {
                    stringify![vec!["bash", self.target.to.to_str().unwrap()]]
                }
            }
        };

        log::debug!("Executor command arguments: {:?}", command_args);

        let mut command = Command::new(&command_args[0]);

        let builder = || {
            command
                .args(&command_args[1..command_args.len()])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .envs(env_vars)
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

    /// Build the context of this executor and return its tag.
    pub fn build_env(&self, signal_receiver: Receiver<bool>, backend: &str) -> Result<String> {
        let mut cleanup: Vec<PathBuf> = vec![];

        if self.env.is_none() {
            bail!("Cannot build environment with no environment file.")
        }

        // Move the executor file
        cleanup.push(self.target.copy()?);
        let containerfile_path = self.env.clone().unwrap();

        let containerfile_name = containerfile_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let env_name: String = containerfile_name.split('.').take(1).collect();
        let env_name = env_name + "_kerblam_runtime";

        let containerfile_path = containerfile_path.as_os_str().to_string_lossy().to_string();

        let builder = || {
            Command::new(backend)
                // If the `self.env` path is not UTF-8 I'll eat my hat.
                .args([
                    "build",
                    "-f",
                    containerfile_path.as_str(),
                    "--tag",
                    env_name.as_str(),
                    ".",
                ])
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
            log::debug!("Failed to build container environment. Unwinding early.");

            for file in cleanup {
                // The idea is that this cleanup should not fail, and anyway
                // we don't really care if it does or not.
                let _ = fs::remove_file(file);
            }

            bail!("Command exited with an error.",);
        };

        // Cleanup now that the build is over
        for file in cleanup {
            let _ = fs::remove_file(file);
        }

        Ok(env_name)
    }

    pub fn create(
        root_path: impl AsRef<Path>,
        executor: impl AsRef<Path>,
        environment: Option<impl AsRef<Path>>,
    ) -> Result<Self> {
        let executor = executor.as_ref();
        let root_path = root_path.as_ref().to_path_buf();
        if !executor.exists() {
            bail!("Executor file {executor:?} does not exist");
        }

        let target_mover = FileMover {
            from: executor.to_path_buf(),
            to: root_path.join("executor"),
        };

        let strategy = match executor.extension() {
            None => {
                return Err(anyhow!("Cannot determine execution strategy"))
                    .with_context(|| "Specified executor has no extension")
            }
            Some(x) => match x.to_str().unwrap() {
                "makefile" => ExecutionStrategy::Make,
                "sh" => ExecutionStrategy::Shell,
                ext => {
                    return Err(anyhow!("Cannot determine execution strategy"))
                        .with_context(|| format!("Unrecognized extension '{ext}'."))
                }
            },
        };

        match environment {
            None => Ok(Self {
                target: target_mover,
                env: None,
                strategy,
            }),
            Some(x) => {
                let x = x.as_ref();
                Ok(Self {
                    target: target_mover,
                    env: Some(x.to_path_buf()),
                    strategy,
                })
            }
        }
    }

    #[allow(dead_code)]
    pub fn strategy(&self) -> ExecutionStrategy {
        self.strategy
    }

    /// Will this executor run in an environment?
    pub fn has_env(&self) -> bool {
        self.env.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionStrategy {
    Make,
    Shell,
}

impl Copy for ExecutionStrategy {}

#[derive(Debug, Clone)]
pub struct FileMover {
    from: PathBuf,
    to: PathBuf,
}

impl FileMover {
    /// Rename the files, destroying this file mover and making the reverse.
    ///
    /// This also updates the last modification time to the present, to
    /// let `make` know that we must regenerate the files.
    pub fn rename(self) -> Result<FileMover> {
        log::debug!("Moving {:?} to {:?}", self.from, self.to);
        fs::rename(&self.from, &self.to)?;
        set_file_mtime(&self.to, FileTime::now())?;

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
    pub fn copy(&self) -> Result<PathBuf> {
        log::debug!("Copying {:?} to {:?}", self.from, self.to);
        fs::copy(self.from.clone(), self.to.clone())?;

        Ok(self.to.clone())
    }

    /// Invert this object, creating a new one with swapped from/to fields.
    ///
    /// Takes ownership of itself, therefore destroying the original.
    pub fn invert(self) -> Self {
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

#[allow(dead_code)]
enum CommandResult {
    Exited { res: ExitStatus },
    Killed,
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
