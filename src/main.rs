use clap::{Parser, Subcommand};
use std::{error::Error, path::PathBuf};
use env_logger;

mod errors;
mod commands;
mod options;
mod utils;

use crate::commands::new;

const KERBLAM_LONG_ABOUT: &str = "Remember, if you want it - Kerblam it!";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author = "hedmad", about = KERBLAM_LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new Kerblam! project.
    New {
        /// Path of the new project.
        path: PathBuf,
    },
    /// Package a Kerblam! project for later
    Pack {
        /// Where to save the packed project
        path: Option<PathBuf>,
    },
    /// Clone a remote git Kerblam! project
    Clone {
        /// The remote git URL to clone
        remote_url: String,
    },
    /// Hydrate an existing Kerblam! project
    Hydrate { path: Option<PathBuf> },
    /// Unpack a packed Kerblam! project
    Unpack {
        packed_path: PathBuf,
        destination: Option<PathBuf>,
    },
    /// Run a Kerblam! project
    Run {
        /// Name of the module to run
        module_name: String,
        /// Path to the Kerblam! project to be run
        project_path: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Cli::parse();

    match args.command {
        Command::New { path } => {
            if let Some(name) = path.file_name() {
                println!("Kerblam! Making new project '{}'", name.to_str().expect("What a weird path!"));
            } else {
                println!("Please provide the name of the project.");
                return Ok(());
            }
            new::create_kerblam_project(&path)?;
        }
        _ => {
            println!("Not implemented yet!")
        }
    };

    Ok(())
}
