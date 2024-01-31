use clap::{Parser, Subcommand};

use anyhow::*;
use options::parse_kerblam_toml;
use std::env::set_current_dir;
use std::{env::current_dir, path::PathBuf};

mod commands;
mod options;
mod utils;

use crate::commands::data;
use crate::commands::new;
use crate::commands::other;
use crate::commands::package;
use crate::commands::run;
use crate::utils::find_kerblam_toml;
use crate::utils::find_pipe_by_name;

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
        /// The label of the exported docker image
        #[arg(long)]
        name: Option<String>,
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
    },
    // Pack local data for export to others
    Pack {
        output_path: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    // 'main' automatically exits with `0` if it gets Ok, and '1' if it gets
    // an Err, also calling `eprintln!(error)` for you.
    // So, we can just return the `StopError` when we get them.
    env_logger::init();

    let args = Cli::parse();
    let toml_file = find_kerblam_toml();

    match toml_file {
        // If we find a toml file, move the current working directory there.
        Some(path) => set_current_dir(path.parent().unwrap())?,
        None => (),
    }
    let here = &current_dir().unwrap();

    log::debug!("Kerblam is starting in {:?}", here);

    let config = parse_kerblam_toml(current_dir().unwrap().join("kerblam.toml"));

    if !matches!(args.command, Command::New { .. }) && config.is_err() {
        return Err(anyhow!("Failed to read the kerblam.toml file!").context(config.unwrap_err()));
    }

    match args.command {
        Command::New { path } => {
            eprintln!("Creating a new project in {:?}!", path);
            new::create_kerblam_project(&path)?;
        }
        Command::Run {
            module_name,
            profile,
            local,
            desc,
        } => {
            let config = config.unwrap(); // This is always safe due to the check above.
            let pipe = find_pipe_by_name(&config, module_name)?;
            if desc {
                eprintln!("{}", pipe.long_description());
                return Ok(());
            }
            run::kerblam_run_project(config, pipe, &current_dir().unwrap(), profile, local)?;
        }
        Command::Data { subcommand } => match subcommand {
            None => {
                let config = config.unwrap();
                let data_info = data::get_data_status(config)?;
                println!("{}", data_info)
            }
            Some(DataCommands::Fetch) => data::fetch_remote_data(config.unwrap())?,
            Some(DataCommands::Clean {
                keep_remote,
                keep_dirs,
            }) => data::clean_data(config.unwrap(), keep_remote, keep_dirs)?,
            Some(DataCommands::Pack { output_path: path }) => data::package_data_to_archive(
                config.unwrap(),
                path.unwrap_or(here.join("data/data_export.tar.gz")),
            )?,
        },
        Command::Package { pipe, name } => {
            let default_pipe_name = format!("{}_exec", &pipe.clone().unwrap_or("x".to_string()));
            let config = config.unwrap();
            let pipe = find_pipe_by_name(&config, pipe)?;
            package::package_pipe(config, pipe, &name.unwrap_or(default_pipe_name))?;
        }
        Command::Ignore {
            path_or_name,
            compress,
        } => {
            other::ignore(&current_dir()?.join(".gitignore"), &path_or_name, compress)?;
        }
    };

    Ok(())
}
