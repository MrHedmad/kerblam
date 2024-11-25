use std::env::{current_dir, set_current_dir};
use std::fs::{create_dir_all, read_to_string, File};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};
use clap::Args;
use tempfile::TempDir;

use crate::cli::Executable;
use crate::execution::{generate_bind_mount_strings, run_protected_command, CommandResult};
use crate::options::{ContainerBackend, KerblamTomlOptions};
use crate::utils::gunzip_file;

/// Replay a pipeline previously packaged with `package`
///
/// The replay tarball made by `kerblam package` contains all the info
/// to replay a workflow run in the past. This can be done manually by
/// anyone with a containerization engine.
///
/// This command is here for convenience: it unpacks the files for you
/// in their correct positions and starts the replay workflow on top
/// with the correct mountpoints.
///
/// Example:
///     > Replay the 'test.kerblam.tar' replay package
///         kerblam replay test.kerblam.tar
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct ReplayCommand {
    /// The name of the compressed replay package
    name: PathBuf,
    /// Where the replay should happen in. Defaults to the current dir
    destination: Option<PathBuf>,
    /// Skip decompressing data? Useful if you replayed before already.
    #[arg(long, short, action)]
    no_decompress: bool,
    /// The name of the container used to run, overriding the
    /// instructions of the .kerblam file.
    #[arg(long, short)]
    tag: Option<String>,
    /// The backend to use, either 'docker' or 'podman'
    #[arg(long, short)]
    #[clap(default_value = "docker")]
    backend: ContainerBackend,
}

impl Executable for ReplayCommand {
    fn execute(self) -> Result<()> {
        replay(
            self.name,
            self.destination
                .unwrap_or(current_dir().expect("Could not find current directory")),
            self.no_decompress,
            self.tag,
            self.backend,
        )?;
        Ok(())
    }
}

/// Replay a kerblam! analysis.
///
/// The 'name' in the path to a .kerb file with the following structure:
/// - (the .kerb file is a .tar file)
/// - the kerblam.toml used by the original analysis;
/// - a tar.gz of the precious data in the input data dir named 'data.tar.gz';
/// - a 'name' file that has just the tag of the container made when
///   'kerblam package' was run.
///
/// This function:
/// - Opens the tar file to a temporary directory;
/// - Reads the kerblam.toml of the original analysis to find the locations
///   of the data directories;
/// - Reads the 'name' file to get the name of the docker to pull
/// - Launches the docker container in the destination folder with
///   the same mountpoints as a generic kerblam run, just a different container
///   name.
pub fn replay(
    name: PathBuf,
    destination: PathBuf,
    no_decompress: bool,
    tag: Option<String>,
    backend: ContainerBackend,
) -> Result<()> {
    let decompression_dir = TempDir::new()?;

    eprintln!("Reading .kerblam tarball...");
    let conn = File::open(name)?;
    let mut archive = tar::Archive::new(conn);

    archive.unpack(&decompression_dir)?;

    let read_name = read_to_string(decompression_dir.path().join("name"));
    let tag_name = match tag {
        Some(x) => x,
        None => {
            if let Ok(name) = read_name {
                name
            } else {
                bail!("Could not read container name. Try running with --tag")
            }
        }
    };

    let package_config: KerblamTomlOptions = toml::from_str(&read_to_string(
        decompression_dir.path().join("kerblam.toml"),
    )?)?;

    let data_archive = decompression_dir.path().join("data.tar.gz");
    if !data_archive.exists() {
        bail!("Data archive not found.")
    };
    eprintln!("Ready to replay! Reconstructing original environment...");
    // For the config files to work, we need to move inside the destination
    create_dir_all(&destination)?;
    set_current_dir(destination)?;

    create_dir_all(package_config.input_data_dir())?;
    create_dir_all(package_config.output_data_dir())?;
    create_dir_all(package_config.intermediate_data_dir())?;

    if !no_decompress {
        log::debug!(
            "Unpacking {data_archive:?} to {:?}",
            package_config.input_data_dir()
        );
        let data = gunzip_file(&data_archive, &data_archive)?;
        let data_archive_conn = File::open(data)?;
        let mut data_archive = tar::Archive::new(data_archive_conn);
        data_archive.unpack(package_config.input_data_dir())?;
    };

    // We can free the decompression dir, we have no longer a need for it
    drop(decompression_dir);

    log::debug!("Calling container backend for execution...");

    let backend: String = backend.into();
    let bind_mounts = generate_bind_mount_strings(&package_config);
    let mut mounts: Vec<String> = vec![];
    for item in bind_mounts {
        mounts.push("-v".to_string());
        mounts.push(item.to_string());
    }

    let mut command = Command::new(backend);
    let builder = || {
        command
            .arg("run")
            .arg("-it")
            .arg("--rm")
            .args(mounts)
            .arg(tag_name)
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Cannot retrieve command output!")
    };

    eprintln!("Replaying...");
    let _return_value = match run_protected_command(builder) {
        Ok(CommandResult::Exited { res }) => Ok(Some(res)), // We don't care if it succeeded.
        Ok(CommandResult::Killed) => {
            eprintln!("\nChild process was killed.");
            Ok(None)
        }
        Err(e) => {
            eprintln!("\nChild process failure: {}\n", e);
            Err(e)
        }
    }?;

    eprintln!("Replay finished!");

    Ok(())
}
