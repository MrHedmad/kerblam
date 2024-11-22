use core::panic;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::io::{stdout, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};

use crate::options::KerblamTomlOptions;
use crate::utils::update_timestamps;

use anyhow::{anyhow, bail, Context, Result};
use crossbeam_channel::{bounded, Receiver};
use lazy_static::lazy_static;

// TODO: I think we can add all cleanup code to `Drop`, so that a lot of these
// functions can be simplified a lot.
// E.g. make a "cleanup: Option<Vec<PathBuf>>" to the Executor and remove
// (without returing the fail) the files specified in the vector (if any)
// so that we can stop at any time and still be sure to cleanup.
// The same idea could be used for the FileRenamers, but we'd need to be
// careful on when they are dropped.

/// Create a recevier that emits `true` when a SIGINT is received
///
/// The receiver queue is of 2 slots. If two signals are sent but not received,
/// the function panics.
/// This is to allow many SIGINTS to actually immediately kill this program
/// if the user really wants to.
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

lazy_static! {
    pub static ref KEYBOARD_INTERRUPT_RECEIVER: Receiver<bool> = setup_ctrlc_hook().unwrap();
}

/// Encapsulate what file to execute and how to execute it
///
/// Fields must be private as they depend on eachother.
pub struct Executor {
    /// A `FileMover` that targets the file to execute.
    /// The analysis will be based on the `to` field.
    target: FileMover,
    /// Optionally, the path to the container file to execute inside of
    env: Option<PathBuf>,
    /// The execution strategy. Depends on which target to execute.
    strategy: ExecutionStrategy,
}

/// Stringify a something that can be iterated upon of non-strings.
macro_rules! stringify {
    ( $x:expr ) => {{
        $x.into_iter().map(|x| x.to_string()).collect()
    }};
}

/// Generate a series of strings suitable for -v options to bind the data dirs
///
/// This generates a binding for each of the input/output/intermediate data
/// dirs and makes a `-v` argument that can be passed to the backend in order
/// to mimick the local file system in the container.
///
/// Returns a vector of strings with the various `-v source:sink` options.
pub fn generate_bind_mount_strings(config: &KerblamTomlOptions) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    let root = current_dir().unwrap();

    let dirs = vec![
        config.input_data_dir(),
        config.output_data_dir(),
        config.intermediate_data_dir(),
    ];

    let host_workdir = match config.execution.workdir.clone() {
        Some(p) => format!("{}", p.to_string_lossy()),
        None => "/".into(),
    };

    log::debug!("Host workdir set to {host_workdir:?}");

    for dir in dirs {
        // the folder here, in the local file system
        let local = dir.to_string_lossy();
        // the folder in the host container
        let host = dir.strip_prefix(&root).unwrap().to_string_lossy();
        let host = format!("{host_workdir}/{host}");

        result.push(format!("{}:{}", local, host))
    }

    log::debug!("Generated bind mount strings: {:?}", result);

    result
}

impl Executor {
    /// Execute this executor based on its data
    ///
    /// Either builds and runs a container, or executes make and/or
    /// bash, depending on the execution strategy.
    ///
    /// Needs the kerblam config to work, as we need to bind-mount the local
    /// paths in the containers as locally needed and follow other settings.
    ///
    /// Destroys itself in the process, as it might change the state of the
    /// filesystem and therefore invalidate itself during execution.
    pub fn execute(
        self,
        config: &KerblamTomlOptions,
        env_vars: HashMap<String, String>,
        skip_build_cache: bool,
        extra_args: Option<Vec<String>>,
    ) -> Result<Option<ExitStatus>> {
        let mut cleanup: Vec<PathBuf> = vec![];

        let mut command_args = if self.env.is_some() {
            // This is a containerized run
            let backend: String = config.execution.backend.clone().into();
            let runtime_name = self.build_env(&backend, skip_build_cache)?;
            let mut partial: Vec<String> = if stdout().is_terminal() {
                // We are in a terminal. Run interactively
                stringify![vec![&backend, "run", "--rm", "-it"]]
            } else {
                // We are not in a terminal. Run normally
                stringify![vec![&backend, "run", "--rm"]]
            };
            // We need to bind-mount the same data dirs as specified in the options
            let mounts = generate_bind_mount_strings(config);
            for mount in mounts {
                partial.extend(vec!["-v".to_string(), mount].into_iter())
            }

            // Add the correct entrypoint override
            let workdir = config.execution.workdir.clone();
            let workdir = match workdir {
                Some(p) => p,
                None => PathBuf::from("/"),
            };
            let workdir = workdir.to_string_lossy();
            let execution_command: Vec<String> = match self.strategy {
                ExecutionStrategy::Make => stringify!(vec![
                    "--entrypoint",
                    "make",
                    &runtime_name,
                    "-f",
                    "executor",
                    "-C",
                    &workdir
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

        if extra_args.is_some() {
            log::debug!("Appending extra command arguments...");
            command_args.extend(extra_args.unwrap());
        }

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

        let return_value = match run_protected_command(builder) {
            Ok(CommandResult::Exited { res }) => Ok(Some(res)), // We don't care if it succeeded.
            Ok(CommandResult::Killed) => {
                eprintln!("\nChild process exited early. Continuing to cleanup...");
                Ok(None)
            }
            Err(e) => {
                eprintln!("\nChild process failure: {}\nContinuing to cleanup...", e);
                Err(e)
            }
        };

        for file in cleanup {
            // The idea is that this cleanup should not fail, and anyway
            // we don't really care if it does or not.
            let _ = fs::remove_file(file);
        }

        return_value
    }

    /// Build the context of this executor and return its tag.
    ///
    /// If the executor has no environment file, this function fails.
    pub fn build_env(&self, backend: &str, no_cache: bool) -> Result<String> {
        let mut cleanup: Vec<PathBuf> = vec![];

        if self.env.is_none() {
            bail!("Cannot build environment with no environment file.")
        }

        // Move the executor file and register it for cleanup
        cleanup.push(self.target.copy()?);
        let containerfile_path = self.env.clone().unwrap();

        let containerfile_name = containerfile_path
            .file_name()
            .unwrap() // Should be safe
            .to_string_lossy()
            .to_string();
        let env_name: String = containerfile_name.split('.').take(1).collect();
        let env_name = env_name + "_kerblam_runtime";

        let containerfile_path = containerfile_path.as_os_str().to_string_lossy().to_string();

        let build_args: Vec<&str> = if no_cache {
            vec![
                "build",
                "-f",
                containerfile_path.as_str(),
                "--tag",
                env_name.as_str(),
                "--no-cache",
                ".",
            ]
        } else {
            vec![
                "build",
                "-f",
                containerfile_path.as_str(),
                "--tag",
                env_name.as_str(),
                ".",
            ]
        };

        let builder = || {
            Command::new(backend)
                // If the `self.env` path is not UTF-8 I'll eat my hat.
                .args(&build_args)
                .stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to spawn builder process.")
        };

        let success = match run_protected_command(builder) {
            Ok(CommandResult::Exited { res }) => res.success(),
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

    /// Create a new executor
    ///
    /// The execution strategy is inferred from the name of the executor file
    /// and if an environment is passed.
    ///
    /// # Arguments
    /// - `root_path`: The folder where this executor will be executed
    /// - `executor`: The file to execute.
    ///   If `*.makefile`, use `ExecutionStrategy::Make`.
    ///   If `*.sh`, use `ExecutionStrategy::Shell`
    /// - `environment`: The containerfile to run this executor with, if any.
    pub fn create(
        root_path: impl AsRef<Path>,
        executor: impl AsRef<Path>,
        environment: Option<impl AsRef<Path>>,
    ) -> Result<Self> {
        let executor = executor.as_ref();
        let root_path = root_path.as_ref().to_path_buf();

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
/// Struct to conveniently move, copy or symlink two files
///
/// Has convenience methods to move files around and undo these moves if needed.
pub struct FileMover {
    from: PathBuf,
    to: PathBuf,
}

impl FileMover {
    /// Rename the files, destroying this file mover and making the reverse.
    ///
    /// Works identically to `mv`, but returns a new `FileMover` that can be
    /// used to quickly undo the move.
    pub fn rename(self, update_time: bool) -> Result<FileMover> {
        log::debug!("Moving {:?} to {:?}", self.from, self.to);
        fs::rename(&self.from, &self.to)?;
        if update_time {
            update_timestamps(&self.to)?;
        }

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
    /// This function destroys this original object.
    pub fn invert(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }

    pub fn get_from(&self) -> PathBuf {
        self.from.clone()
    }
    #[allow(dead_code)]
    pub fn get_to(&self) -> PathBuf {
        self.to.clone()
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

pub enum CommandResult {
    Exited { res: ExitStatus },
    Killed,
}

/// Run a command but keep listening to events.
///
/// Takes whatever command is executed by `cmd_builder` and runs it.
/// While the child is running, listens every 50ms to `receiver` for a `true` signal.
/// Upon receiving such a signal, kill the child early and return.
///
/// Useful to catch external events such as SIGKILL or SIGINT during the
/// execution of a command and redirect them to kill the children, and not
/// the currently running process.
///
/// Returns an error if the child cannot be killed.
/// Panics if something really bad happens and the kernel cannot get a handle
/// on what the child is doing.
pub fn run_protected_command<F>(cmd_builder: F) -> Result<CommandResult>
where
    F: FnOnce() -> Child,
{
    let mut child = cmd_builder();
    let receiver = KEYBOARD_INTERRUPT_RECEIVER.clone();

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
