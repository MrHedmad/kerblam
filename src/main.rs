use clap::{Parser, Subcommand};
use std::{
    error::Error,
    path::PathBuf,
    str::FromStr,
};

mod errors;
mod new;
mod utils;
mod options;

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
        path: String,
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

fn create_kerblam_project(dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let dirs_to_create: Vec<&str> = vec!["", "./data/in", "./data/out", "./src/modules"];
    // Ask for user input
    // I defined `dirs_to_create` before so that if we ever have to add to them
    // dynamically we can do so here.
    let use_python = utils::UserResponse::ask_options(
        "Do you need Python?",
        concat!(
            "Will you use Python in this project?\n",
            "If so, I'll .gitignore python's files and make a virtual environment."
        ),
    );
    let use_r = utils::UserResponse::ask_options(
        "Do you need R?",
        concat!(
            "Will you use R in this project?\n",
            "If so, I'll add R to the .gitignore."
        ),
    );

    let paths_to_create: Vec<PathBuf> = dirs_to_create
        .into_iter()
        .map(|x| dir.clone().join(x))
        .collect();

    let results: Vec<Result<String, String>> = paths_to_create
        .iter()
        .map(|x| utils::kerblam_create_dir(x))
        .collect();
    let mut stop = false;
    for res in results {
        match res {
            Ok(msg) => println!("{}", msg),
            Err(msg) => {
                println!("{}", msg);
                stop = true;
            }
        }
    }

    match utils::kerblam_create_file(
        &dir.join("./kerblam.toml"),
        format!("[meta]\nversion: {}", VERSION).as_str(),
        true,
    ) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => {
            println!("{}", msg);
            stop = true
        }
    }

    // This is kind of useless... for now?
    if stop {
        return Ok(());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        Command::New { path } => {
            create_kerblam_project(&PathBuf::from_str(path.as_str())?)?;
        }
        _ => {
            println!("Not implemented yet!")
        }
    };

    Ok(())
}
