use std::env::set_current_dir;
use std::io::Write;

use std::{
    env::{current_dir, current_exe},
    fs::{copy, create_dir_all, File},
    process::{Command, Stdio},
};

use crate::execution::Executor;
use crate::options::KerblamTomlOptions;
use crate::options::Pipe;
use crate::utils::{find_files, gzip_file, tar_files};

use anyhow::{bail, Result};
use serde::Serialize;

use tempfile::tempdir;

#[derive(Serialize, Debug)]
struct Signature {
    name: String,
    email: String,
    on: String,
    // Perhaps add RSA key?
}

/// Gets a git option given a name
fn get_git_option(option: &str) -> Result<String> {
    match Command::new("git").args(vec!["config", option]).output() {
        Ok(output) => {
            let msg = String::from_utf8(output.stdout).expect("Could not parse output from UTF8");
            if output.status.success() {
                return Ok(msg.trim().to_owned());
            } else {
                bail!(msg)
            }
        }
        Err(_) => bail!("Could not execute command"),
    }
}

impl Signature {
    fn new() -> Result<Signature> {
        let git_name = get_git_option("user.name")?;
        let git_email = get_git_option("user.email")?;
        let current_time = chrono::Utc::now().to_string();

        Ok(Signature {
            name: git_name,
            email: git_email,
            on: current_time,
        })
    }
}

/// Package a pipeline for execution later
///
/// # Arguments
///
/// - `config`: The kerblam config for this execution.
/// - `pipe`: The name of the pipe to execute
/// - `package_name`: The name of the container image built by this execution.
pub fn package_pipe(
    config: KerblamTomlOptions,
    pipe: Pipe,
    package_name: &str,
    include_signature: bool,
) -> Result<()> {
    let pipe_name = pipe.name();
    log::debug!("Packaging pipe {pipe_name} as {package_name}...");
    let here = current_dir()?;

    let precious_files = config.precious_files();
    let input_data_dir = config.input_data_dir();

    // We have to purge all data from the context, so that we can package it
    // separately. For this reason we make a temporary context without it,
    // and build the container in there.
    log::debug!("Starting to build temporary context...");
    let temp_build_dir = tempfile::tempdir()?;
    log::debug!("Temporary directory: {temp_build_dir:?}");
    log::debug!("Copying context without data...");
    let context_files = find_files(
        &here,
        Some(vec![
            config.input_data_dir(),
            config.output_data_dir(),
            config.intermediate_data_dir(),
            config.env_dir(),
        ]),
    );
    for file in context_files {
        log::debug!("Adding {file:?} to temporary context.");
        let target = temp_build_dir.path().join(file.strip_prefix(&here)?);
        create_dir_all(target.parent().unwrap())?;
        copy(&file, target)?;
    }

    // We now start from this new context and build the original container.
    // We must work in the temporary directory, however.
    set_current_dir(temp_build_dir.path())?;

    log::debug!("Building initial context...");
    let executor: Executor = pipe.into_executor(&here)?;
    let myself = current_exe()?;

    if !executor.has_env() {
        bail!(
            "Cannot proceed! Pipe {:?} has no related container_file.",
            pipe_name
        )
    };
    let backend: String = config.execution.backend.clone().into();
    let base_container = executor.build_env(&backend, false)?;
    log::debug!("Base container name: {base_container:?}");

    // We now have the empty container. We can add our own layers.
    log::debug!("Writing wrapper container file.");

    // Write the container_file
    let workdir = config.execution.workdir.clone();
    let workdir = match workdir {
        Some(p) => format!("{}", p.to_string_lossy()),
        None => "/".into(),
    };
    // Copy the kerblam! executable here, so we can include it in the context.
    log::debug!("Copying kerblam! executable to context...");
    let kerblam_path = temp_build_dir.path().join("kerblam");
    copy(myself, kerblam_path)?;

    let content = format!("FROM {base_container}\nWORKDIR {workdir}\nCOPY ./kerblam .\nENTRYPOINT [\"./kerblam\", \"run\", \"{pipe_name}\"]");
    log::debug!("Execution string: {content}");
    let new_container_file_path = temp_build_dir.path().join("Containerfile");
    let mut new_container_file = File::create(&new_container_file_path)?;
    new_container_file.write_all(content.as_bytes())?;

    log::debug!("Packaging container...");
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
        println!(
            "✅ Packaged pipe container ({}) as {}!",
            pipe_name, &package_name
        );
    } else {
        bail!("Failed to package pipe!")
    };

    // We now have to export the data.
    let temp_package = tempdir()?;
    log::debug!("Packaging temporary directory: {temp_package:?}");
    let data_package = tar_files(
        precious_files,
        input_data_dir,
        temp_package.path().join("data"),
    )?;
    let data_package = gzip_file(&data_package, &data_package)?;

    // Create the 'name' file
    let name_file = temp_package.path().join("name");
    let mut name_file_conn = File::create(name_file)?;
    write!(name_file_conn, "{}", package_name)?;

    let package = here.join(format!("{}.kerblam.tar", pipe_name));
    let package_conn = File::create(&package)?;
    let mut package_archive = tar::Builder::new(package_conn);

    package_archive.append_path_with_name(here.join("kerblam.toml"), "kerblam.toml")?;
    package_archive
        .append_path_with_name(&data_package, data_package.strip_prefix(&temp_package)?)?;
    package_archive.append_path_with_name(&temp_package.path().join("name"), "name")?;

    // Create the 'signature' file
    // This inherits from the git config files
    if include_signature {
        let signature_file = temp_package.path().join("signature.json");
        let mut signature_file_conn = File::create(signature_file)?;
        let signature = Signature::new()?;

        write!(
            signature_file_conn,
            "{}",
            serde_json::to_string(&signature).unwrap()
        )?;
        package_archive.append_path_with_name(
            &temp_package.path().join("signature.json"),
            "signature.json",
        )?;
    }

    eprintln!("✅ Created replay package at {:?}!", package);

    drop(temp_package);

    Ok(())
}
