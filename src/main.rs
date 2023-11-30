use clap::{Parser, Subcommand};

use anyhow::*;
use options::parse_kerblam_toml;
use std::{env::current_dir, path::PathBuf};

mod commands;
mod options;
mod utils;

use crate::commands::data;
use crate::commands::new;
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
    /// Clone a remote git Kerblam! project
    Clone {
        /// The remote git URL to clone
        remote_url: String,
    },
    /// Run a Kerblam! project
    Run {
        /// Name of the module to run
        module_name: String,
        /// Optional data profile to run with
        #[arg(long)]
        profile: Option<String>,
    },
    /// Get information on local data and manage it
    Data {
        #[command(subcommand)]
        subcommand: Option<DataSubcommand>,
    },
}

#[derive(Subcommand, Debug, PartialEq, Clone)]
enum DataSubcommand {
    Clean {},
    Package {
        /// Path to save packaged output
        path: Option<PathBuf>,
    },
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
        Command::Data { subcommand } => {
            let config = config.unwrap();
            match subcommand {
                None => data::print_data_status(config, current_dir().unwrap())?,
                Some(DataSubcommand::Clean {}) => {
                    data::cleanup_data(config, current_dir().unwrap())?
                }
                Some(DataSubcommand::Package { path }) => {
                    let here = current_dir().unwrap();
                    data::package_data(
                        config,
                        here.clone(),
                        path.unwrap_or(here.join("exported_data.tar.gz")),
                    )?
                }
            };
        }

        _ => {
            todo!()
        }
    };

    Ok(())
}
