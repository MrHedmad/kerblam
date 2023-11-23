use clap::{Parser, Subcommand};
use env_logger;
use options::parse_kerblam_toml;
use std::{env::current_dir, error::Error, path::PathBuf};

mod commands;
mod errors;
mod options;
mod utils;

use crate::commands::new;
use crate::commands::run;

const KERBLAM_LONG_ABOUT: &str = "Remember, if you want it - Kerblam it!";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
        /// Optional data profile to run with
        #[arg(long)]
        profile: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    // The failure cases should return an error code of != 0, and the println!
    // should probably print to the stderr. Instead of passing `Ok(())`, you
    // might be able to do the following...
    env_logger::init();
    let args = Cli::parse();

    let config = parse_kerblam_toml(&current_dir().unwrap());

    if matches!(args.command, Command::New { .. }) && config.is_err() {
        // We cannot go forward with any command if we are not in
        // a kerblam! project.
        // TODO: Maybe we could check parent directories for a kerblam.toml
        // file and run as if we were there.
        return Err("❓ Could not find a 'kerblam.toml' in the current WD. Abort!".into());
    }

    let config = config.unwrap(); // This is always safe due to the check above.

    match args.command {
        Command::New { path } => {
            if let Some(name) = path.file_name() {
                eprintln!(
                    "Kerblam! Making new project '{}'",
                    name.to_str().expect("What a weird path!")
                );
            } else {
                return Err("Please provide the name of the project.".into());
            }
            new::create_kerblam_project(&path)?;
        }
        Command::Run {
            module_name,
            profile,
        } => {
            match run::kerblam_run_project(config, module_name, &current_dir().unwrap(), profile) {
                Ok(msg) => println!("{}", msg),
                Err(err) => println!("❌ {}", err.msg),
            };
        }
        _ => {
            unimplemented!("This command is not implemented yet!")
        }
    };

    Ok(())
}
