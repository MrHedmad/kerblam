use std::io::Write;
use std::{
    env::{current_dir, current_exe},
    fs::{copy, create_dir_all, File},
    process::{Command, Stdio},
};

use crate::options::Pipe;
use crate::utils::find_files;
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
/// - `package_name`: The name of the container image built by this execution.
pub fn package_pipe(config: KerblamTomlOptions, pipe: Pipe, package_name: &str) -> Result<()> {
    let pipe_name = pipe.name();
    log::debug!("Packaging pipe {pipe_name} as {package_name}...");
    let here = current_dir()?;

    // Create an executor for later.
    let executor: Executor = pipe.to_executor(&here)?;
    let myself = current_exe()?;

    if !executor.has_env() {
        bail!(
            "Cannot proceed! Pipe {:?} has no related container_file.",
            pipe_name
        )
    };

    // We have to setup the directory to be ready to be executed.
    // We already do this with the executors, so we can just borrow that from
    // the `run` command...

    log::debug!("Building initial context...");
    let sigint_rec = setup_ctrlc_hook().expect("Failed to setup SIGINT hook!");
    let backend: String = config.execution.backend.clone().into();
    let base_container = executor.build_env(sigint_rec, &backend)?;
    log::debug!("Base container name: {base_container:?}");

    // We now start from this new container and add our own layers, copying the
    // precious files and more from the - not ignored - context.
    // We must work in a temporary directory, however.
    log::debug!("Starting to build temporary context...");
    let temp_build_dir = tempfile::tempdir()?;
    log::debug!("Temporary directory: {temp_build_dir:?}");
    let precious_files = config.precious_files();
    log::debug!("Files deemed precious: {precious_files:?}");
    for file in precious_files {
        log::debug!("Adding {file:?} to temporary context.");
        let target = temp_build_dir.path().join(file.strip_prefix(&here)?);
        create_dir_all(target.parent().unwrap())?;
        copy(&file, target)?;
    }
    log::debug!("Copying context...");
    let context_files = find_files(
        &here,
        Some(vec![
            config.input_data_dir(),
            config.output_data_dir(),
            config.temporary_data_dir(),
            config.intermediate_data_dir(),
        ]),
    );
    for file in context_files {
        log::debug!("Adding {file:?} to temporary context.");
        let target = temp_build_dir.path().join(file.strip_prefix(&here)?);
        create_dir_all(target.parent().unwrap())?;
        copy(&file, target)?;
    }

    log::debug!("Copying kerblam! executable to context...");
    copy(myself, temp_build_dir.path().join("kerblam"))?;

    log::debug!("Writing wrapper container_file.");

    // Write the container_file
    let workdir = config.execution.workdir.clone();
    let workdir = workdir.to_string_lossy();
    let content = match executor.strategy() {
        ExecutionStrategy::Make => format!(
            "FROM {base_container}\nCOPY . .\nENTRYPOINT [\"bash\", \"-c\", \"{workdir}/kerblam data fetch && make -C {workdir} -f {workdir}/executor\"]"
        ),
        ExecutionStrategy::Shell => format!(
            "FROM {base_container}\nCOPY . .\nENTRYPOINT [\"bash\", \"-c\", \"{workdir}/kerblam data fetch && bash {workdir}/executor\"]"
        ),
    };
    log::debug!("Execution string: {content}");
    let new_container_file_path = temp_build_dir.path().join("Containerfile");
    let mut new_container_file = File::create(&new_container_file_path)?;
    new_container_file.write_all(content.as_bytes())?;

    log::debug!("Packaging...");
    // Build this new container and tag it...
    let res = Command::new(&backend)
        .args([
            "build",
            "-f",
            &new_container_file_path.to_string_lossy(),
            "--no-cache",
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
        println!("✅ Packaged pipe ({}) as {}!", pipe_name, &package_name);
        Ok(())
    } else {
        bail!("Failed to package pipe!")
    }
}
