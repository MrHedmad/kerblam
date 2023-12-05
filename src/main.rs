use clap::{Parser, Subcommand};

use anyhow::*;
use options::parse_kerblam_toml;
use std::{env::current_dir, path::PathBuf};

mod commands;
mod options;
mod utils;

use crate::commands::data;
use crate::commands::new;
use crate::commands::package;
use crate::commands::run;

const KERBLAM_LONG_ABOUT: &str = "Remember, if you want it - Kerblam it!";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author = "hedmad", about = KERBLAM_LONG_ABOUT)]
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
        module_name: String,
        /// Optional data profile to run with
        #[arg(long)]
        profile: Option<String>,
    },
    /// Manage local data
    Data {
        #[command(subcommand)]
        subcommand: Option<DataCommands>,
    },
    /// Package for execution later
    Package {
        /// The name of the pipe to package
        pipe: String,
        /// The label of the exported docker image
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
enum DataCommands {
    Fetch,
    Clean,
    Pack { output_path: Option<PathBuf> },
}

fn main() -> anyhow::Result<()> {
    // 'main' automatically exits with `0` if it gets Ok, and '1' if it gets
    // an Err, also calling `eprintln!(error)` for you.
    // So, we can just return the `StopError` when we get them.
    env_logger::init();
    let here = &current_dir().unwrap();
    let args = Cli::parse();

    log::debug!("Kerblam is starting in {:?}", here);

    let config = parse_kerblam_toml(current_dir().unwrap().join("kerblam.toml"));

    if !matches!(args.command, Command::New { .. }) && config.is_err() {
        // We cannot go forward with any command if we are not in
        // a kerblam! project.
        // TODO: Maybe we could check parent directories for a kerblam.toml
        // file and run as if we were there.
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
        } => {
            let config = config.unwrap(); // This is always safe due to the check above.
            run::kerblam_run_project(config, module_name, &current_dir().unwrap(), profile)?;
        }
        Command::Data { subcommand } => match subcommand {
            None => {
                let config = config.unwrap();
                let data_info =
                    data::get_data_status(config, &current_dir().unwrap().join("data"))?;
                println!("{}", data_info)
            }
            Some(DataCommands::Fetch) => data::fetch_remote_data(config.unwrap())?,
            Some(DataCommands::Clean) => data::clean_data(config.unwrap())?,
            Some(DataCommands::Pack { output_path: path }) => data::package_data_to_archive(
                config.unwrap(),
                path.unwrap_or(here.join("data/data_export.tar.gz")),
            )?,
        },
        Command::Package { pipe, name } => {
            package::package_pipe(
                config.unwrap(),
                &pipe,
                &name.unwrap_or(format!("{}_exec", &pipe)),
            )?;
        }
    };

    Ok(())
}
