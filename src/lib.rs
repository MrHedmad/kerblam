use clap::{Parser, Subcommand};

use anyhow::*;
use options::{parse_kerblam_toml, ContainerBackend};
use std::env::set_current_dir;
use std::ffi::OsString;
use std::{env::current_dir, path::PathBuf};

mod cache;
mod commands;
mod execution;
mod options;
mod utils;

use commands::{
    clean_data, create_kerblam_project, fetch_remote_data, get_data_status, ignore,
    kerblam_run_project, package_data_to_archive, package_pipe, replay,
};

use crate::utils::find_pipe_by_name;
use crate::utils::{find_kerblam_toml, print_md};

const KERBLAM_LONG_ABOUT: &str = "Remember, if you want it - Kerblam it!";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author = "hedmad", version, about = KERBLAM_LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    /// Initialize a new Kerblam! project.
    New {
        /// Path of the new project.
        path: PathBuf,
    },
    /// Run a Kerblam! project
    Run {
        /// Name of the module to run
        module_name: Option<String>,
        /// Optional data profile to run with
        #[arg(long)]
        profile: Option<String>,
        /// Show pipe description and exit
        #[arg(long, short, action)]
        desc: bool,
        /// Do not run in container even if a container is available
        #[arg(long, short, action)]
        local: bool,
        /// Skip using build cache if running in containers.
        #[arg(long = "no-build-cache", action)]
        skip_build_cache: bool,
        /// Command line arguments to be passed to child process
        #[clap(last = true, allow_hyphen_values = true)]
        extra_args: Option<Vec<String>>,
    },
    /// Manage local data
    Data {
        #[command(subcommand)]
        subcommand: Option<DataCommands>,
    },
    /// Package for execution later
    Package {
        /// The name of the pipe to package
        pipe: Option<String>,
        /// The label of the exported container image
        #[arg(long)]
        tag: Option<String>,
    },
    /// Add paths and languages to .gitignore
    Ignore {
        /// The name of a language or a path to ignore
        ///
        /// Supported names can be seen in https://github.com/github/gitignore
        /// The input is case-sensitive!
        path_or_name: String,
        /// Should the gitignore be compressed?
        #[arg(long, action)]
        compress: bool,
    },
    /// Replay a pipeline packaged with `package`
    Replay {
        /// The name of the compressed kerblam package
        name: PathBuf,
        /// Where the replay should happen in. Defaults to the current
        /// environment
        destination: Option<PathBuf>,
        /// Skip decompressing data? Useful if you replayed before already.
        #[arg(long, short, action)]
        no_decompress: bool,
        /// The name of the container used to unpack with, overriding the
        /// instructions of the .kerblam file.
        #[arg(long, short)]
        tag: Option<String>,
        /// The backend to use, either 'docker' or 'podman'
        #[arg(long, short)]
        #[clap(default_value = "docker")]
        backend: ContainerBackend,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
enum DataCommands {
    /// Fetch remote data and save it locally
    Fetch,
    /// Clean non-essential data to save disk space
    Clean {
        #[arg(long, short, action)]
        /// Do not delete locally present remote files.
        keep_remote: bool,
        #[arg(long, short('d'), action)]
        /// Do not delete locally present directories.
        keep_dirs: bool,
        #[arg(long, short, action)]
        /// Do not ask for any confirmation.
        yes: bool,
    },
    // Pack local data for export to others
    Pack {
        output_path: Option<PathBuf>,
    },
}

/// Run Kerblam! with a certain arguments list.
pub fn kerblam<I, T>(arguments: I) -> anyhow::Result<()>
where
    I: Iterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let here = current_dir()?;
    log::debug!("Kerblam! invoked in {here:?}");
    let args = Cli::parse_from(arguments);
    log::debug!("Args: {args:?}");

    if let Command::New { path } = args.command {
        eprintln!("Creating a new project in {:?}!", path);
        create_kerblam_project(&path)?;
        return Ok(());
    };

    if let Command::Replay {
        name,
        destination,
        no_decompress,
        tag,
        backend,
    } = args.command
    {
        replay(
            name,
            destination.unwrap_or(here),
            no_decompress,
            tag,
            backend,
        )?;
        return Ok(());
    }

    let toml_file = match find_kerblam_toml() {
        // If we find a toml file, move the current working directory there.
        Some(path) => {
            set_current_dir(path.parent().unwrap())?;
            path
        }
        None => {
            bail!(
                "Not a kerblam! project (or any of the parent directories): no kerblam.toml found."
            );
        }
    };

    let here = &current_dir().unwrap();

    log::debug!("Kerblam is starting in {:?}", here);

    let config = parse_kerblam_toml(toml_file)?;

    match args.command {
        Command::New { .. } => unreachable!("This case was covered above already"),
        Command::Run {
            module_name,
            profile,
            local,
            desc,
            skip_build_cache,
            extra_args,
        } => {
            let pipe = find_pipe_by_name(&config, module_name)?;
            if desc {
                print_md(&pipe.long_description());
                return Ok(());
            }
            kerblam_run_project(
                config,
                pipe,
                &current_dir().unwrap(),
                profile,
                local,
                skip_build_cache,
                extra_args,
            )?;
        }
        Command::Data { subcommand } => match subcommand {
            None => {
                let data_info = get_data_status(config)?;
                println!("{}", data_info)
            }
            Some(DataCommands::Fetch) => fetch_remote_data(config)?,
            Some(DataCommands::Clean {
                keep_remote,
                keep_dirs,
                yes,
            }) => clean_data(config, keep_remote, keep_dirs, yes)?,
            Some(DataCommands::Pack { output_path: path }) => package_data_to_archive(
                config,
                path.unwrap_or(here.join("data/data_export.tar.gz")),
            )?,
        },
        Command::Package { pipe, tag } => {
            let default_pipe_name = format!("{}_exec", &pipe.clone().unwrap_or("x".to_string()));
            let pipe = find_pipe_by_name(&config, pipe)?;
            package_pipe(config, pipe, &tag.unwrap_or(default_pipe_name))?;
        }
        Command::Ignore {
            path_or_name,
            compress,
        } => {
            let gitignore_path = here.join(".gitignore");
            ignore(gitignore_path, &path_or_name, compress)?;
        }
        Command::Replay { .. } => {
            unreachable!("This case was covered above already.")
        }
    };

    Ok(())
}
