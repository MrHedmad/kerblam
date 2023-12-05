use std::io::Write;
use std::{
    env::{current_dir, current_exe},
    fs::{copy, create_dir_all, File},
    process::{Command, Stdio},
};

use crate::{
    commands::run::{setup_ctrlc_hook, ExecutionStrategy, Executor},
    options::KerblamTomlOptions,
};

use anyhow::{bail, Result};

/// Package a pipeline for execution later
///
/// # Arguments
///
/// - `config`: The kerblam config for this execution.
/// - `pipe`: The name of the pipe to execute
/// - `package_name`: The name of the docker image built by this execution.
pub fn package_pipe(config: KerblamTomlOptions, pipe: &str, package_name: &str) -> Result<()> {
    log::debug!("Packaging pipe {pipe} as {package_name}");
    let here = current_dir()?;
    // We cannot go on if we are not in a dockerized environment
    let executor = Executor::create(&here, pipe)?;
    let myself = current_exe()?;

    if !executor.has_env() {
        bail!("Cannot proceed! Pipe {} has no related dockerfile.", pipe)
    };

    // We have to setup the directory to be ready to be executed.
    // We already do this with the executors, so we can just borrow that from
    // the `run` command...

    log::debug!("Building initial context...");
    let sigint_rec = setup_ctrlc_hook().expect("Failed to setup SIGINT hook!");
    let base_container = executor.build_env(sigint_rec)?;

    // We now start from this new docker and add our own layers, copying the
    // precious files and more from the - not ignored - context.
    // We must work in a temporary directory, however.
    log::debug!("Starting to build temporary context...");
    let temp_build_dir = tempfile::tempdir()?;
    let precious_files = config.precious_files();
    for file in precious_files {
        log::debug!("Adding {file:?} to temporary context.");
        let target = temp_build_dir.path().join(file.strip_prefix(&here)?);
        create_dir_all(target.parent().unwrap())?;
        copy(&file, target)?;
    }

    log::debug!("Copying kerblam! executable to context...");
    copy(&myself, temp_build_dir.path().join("kerblam"))?;

    log::debug!("Writing wrapper dockerfile.");

    // Write the dockerfile
    let content = match executor.strategy() {
        ExecutionStrategy::Make => format!(
            "FROM {}\nCOPY . .\nCMD kerblam fetch && make .",
            base_container
        ),
        ExecutionStrategy::Shell => format!(
            "FROM {}\nCOPY . .\nCMD kerblam fetch && bash executor",
            base_container
        ),
    };
    let mut new_dockerfile = File::create(temp_build_dir.path().join("Dockerfile"))?;
    new_dockerfile.write_all(content.as_bytes())?;

    log::debug!("Packaging...");
    // Build this new container and tag it...
    let res = Command::new("docker")
        .args([
            "build",
            "--tag",
            package_name,
            &temp_build_dir.path().as_os_str().to_string_lossy(),
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Could not launch program!");

    if res.status.success() {
        println!("✅ Packaged pipe ({}) as {}!", &pipe, &package_name);
        Ok(())
    } else {
        bail!("Failed to package pipe!")
    }
}
