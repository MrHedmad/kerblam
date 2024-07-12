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

const KERBLAM_LONG_ABOUT: &str = concat!(
    "  _  __           _     _               _ \n",
    " | |/ / ___  _ _ | |__ | | __ _  _ __  | |\n",
    " | ' < / -_)| '_|| '_ \\| |/ _` || '  \\ |_|\n",
    " |_|\\_\\\\___||_|  |_.__/|_|\\__,_||_|_|_|(_)\n",
    "                                          \n",
    "Manage your data analysis projects quickly and easily.\n\n",
    "Kerblam! is a project manager that lets you focus on getting things done,\n",
    "with it taking care of tedious or tricky parts of project management.\n\n",
    "You can find the Kerblam! manual online at https://kerblam.dev/.\n",
    "The source code is available at https://github.com/MrHedmad/kerblam"
);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author = "hedmad", version, about = KERBLAM_LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    /// Initialize a new, empty Kerblam! project.
    ///
    /// This command asks you a few initialization questions to get you
    /// started, such as which programming languages you will use, if you
    /// want to use `git` and a remote Git server, etc...
    ///
    /// Examples:
    ///     > Create a new project in the 'my_project' directory
    ///          kerblam new ./my_project
    ///
    #[command(verbatim_doc_comment)]
    New {
        /// Path to the new project
        path: PathBuf,
    },
    /// Start a workflow within a Kerblam! project
    ///
    /// Start workflow managers for your workflows, potentially with a data
    /// profile attached.
    ///
    /// If no workflow is specified, shows the list of available workflows.
    ///
    /// Examples:
    ///     > List the available workflows that Kerblam! can manage
    ///         kerblam run
    ///
    ///     > Run the workflow named 'process_csv.sh'
    ///         kerblam run process_csv
    ///
    ///     > Use the 'test' profile with a workflow
    ///         kerblam run process_csv --profile test
    #[command(verbatim_doc_comment)]
    Run {
        /// Name of the workflow to be started
        module_name: Option<String>,
        /// Name of a data profile to use during this execution
        #[arg(long)]
        profile: Option<String>,
        /// Show the pipe description and exit
        #[arg(long, short, action)]
        desc: bool,
        /// Do not run in container, even if a container is available
        #[arg(long, short, action)]
        local: bool,
        /// Do not use the containerization engine build cache if running in a container
        #[arg(long = "no-build-cache", action)]
        skip_build_cache: bool,
        /// Command line arguments to be passed to child process
        #[clap(last = true, allow_hyphen_values = true)]
        extra_args: Option<Vec<String>>,
    },
    /// Show, fetch, delete or manage local data
    ///
    /// You can measure how large a project is on your disk, bulk fetch
    /// required data from your project, delete unneeded data or package it
    /// up to share it with others.
    ///
    /// If no subcommand is specified, shows the status of current data.
    ///
    /// Examples:
    ///     > Show the amount of data present locally
    ///         kerblam data
    ///
    ///     > Fetch remote data as specified in kerblam.toml
    ///         kerblam data fetch
    ///
    ///     > Clean all non-essential data for this project
    ///         kerblam data clean --delete-remote
    #[command(verbatim_doc_comment)]
    Data {
        #[command(subcommand)]
        subcommand: Option<DataCommands>,
    },
    /// Package a workflow for execution later
    ///
    /// Package your workflow in a container primed for execution and
    /// a replay tarball with input data and execution parameters.
    ///
    /// If you upload the container to a registry and share your replay
    /// tarball, other people can use Kerblam! to re-execute your workflow
    /// (or do it manually).
    ///
    /// Example:
    ///   kerblam package process_csv --tag username/process_csv:latest
    #[command(verbatim_doc_comment)]
    Package {
        /// The name of the workflow to package. Must have a related container
        pipe: Option<String>,
        /// The label of the exported container image
        #[arg(long)]
        tag: Option<String>,
    },
    /// Add paths and whole languages to a .gitignore file
    ///
    /// Supported names can be seen in https://github.com/github/gitignore
    /// The input is case-sensitive!
    ///
    /// Examples:
    ///     > Use the Python gitignore from Github
    ///         kerblam ignore Python
    ///
    ///     > Use the Rust gitignore from Github, and delete duplicates
    ///         kerblam ignore Rust --compress
    ///
    ///     > Ignore a specific file in the project
    ///         kerblam ignore ./src/test_script.sh
    #[command(verbatim_doc_comment)]
    Ignore {
        /// The name of a language or a path to ignore
        path_or_name: String,
        /// Should the gitignore be compressed to remove duplicates?
        #[arg(long, action)]
        compress: bool,
    },
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
    #[command(verbatim_doc_comment)]
    Replay {
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
    },
}

#[derive(Subcommand, Debug, PartialEq)]
enum DataCommands {
    /// Fetch remote data and save it locally
    ///
    /// You can specify data to fetch in the kerblam.toml file, like so:
    ///
    /// [data.remote]
    /// "url to be fetched" = "target file"
    #[command(verbatim_doc_comment)]
    Fetch,
    /// Clean non-essential data to save disk space
    ///
    /// This removes:
    ///     - Output data (in the output directory);
    ///     - Intermediate data (in the intermediate data directory);
    ///     - Data that can be downloaded remotely (in the input data directory);
    /// and all empty directories that are left behind after deletion.
    ///
    /// Examples:
    ///     > Delete everything that is not precious
    ///         kerblam data clean
    ///
    ///     > Delete everything but remote data
    ///         kerblam data clean --keep-remote
    ///
    ///     > Skip the confirmation prompt
    ///         kerblam data clean --yes
    #[command(verbatim_doc_comment)]
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
    /// Pack local data for export to others
    ///
    /// This creates a tarball with the local-only input data and the output
    /// data currently present on disk.
    ///
    /// The created file will always be a tar.gz file, regardless of the name
    /// you give it.
    ///
    /// Example:
    ///      kerblam data pack my_last_execution.tar.gz
    #[command(verbatim_doc_comment)]
    Pack { output_path: Option<PathBuf> },
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
