use clap::{Parser, Subcommand};
use utils::{YesNo, GitCloneMethod};
use std::{error::Error, path::PathBuf, str::FromStr};
use reqwest;

mod errors;
mod new;
mod options;
mod utils;

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

fn fetch_gitignore(name: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://raw.githubusercontent.com/github/gitignore/main/{}.gitignore", name);
    
    let response = reqwest::blocking::get(url)?.text()?;
    Ok(response)
}

fn create_kerblam_project(dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let dirs_to_create: Vec<&str> = vec!["", "./data/in", "./data/out", "./src/pipes", "./src/dockerfiles"];
    let mut files_to_create: Vec<(&str, String)> = vec![("./kerblam.toml", format!("[meta]\nversion = {}", VERSION))];
    let mut commands_to_run: Vec<(&str, Vec<String>)> = vec![];
    let mut gitignore_content: Vec<String> = vec![];
    // Ask for user input
    // I defined `dirs_to_create` before so that if we ever have to add to them
    // dynamically we can do so here.
    if utils::ask_for::<YesNo>(
        "Do you need Python?"
    ).into() {
        // I was once using Vec<(&str, Vec<&str>)> for `commands_to_run`, but
        // PathBuf can become a `String`, and when you `.as_str()`, the original
        // String is freed at the end of this scope, thus rendering the resulting
        // &str reference invalid! The borrow checker complains, so the easiest
        // way that I could think of was to just change the signature to
        // Vec<(&str, Vec<String>)>.
        //
        // Something something generic could probably fix this much more cleanly
        // Or maybe a box?
        commands_to_run.push(("python", vec!["-m", "venv", &dir.join("env").to_str().unwrap()].into_iter().map(|x| x.to_string()).collect()));
        gitignore_content.push(fetch_gitignore("Python").expect("Failed to fetch Python's gitignore.")); 
    };

    if utils::ask_for::<YesNo>(
        "Do you need R?"
    ).into() {
        gitignore_content.push(fetch_gitignore("R").expect("Failed to fetch R's gitignore"));
    }

    if utils::ask_for::<YesNo>(
        "Do you want to use pre-commit?"
    ).into() {
        files_to_create.push(("./pre-commit-config.yaml", String::from("")));
        commands_to_run.push(("pre-commit", vec!["install", "--hook-type", "pre-commit", "--hook-type", "commit-msg"].into_iter().map(|x| x.to_string()).collect()));
    }

    if utils::ask_for::<YesNo>(
        "Do you want to setup the remote origin of the project?"
    ).into() {
        let username = utils::ask("Enter your username: ")?;
        let origin_url = match utils::ask_for::<GitCloneMethod>("What cloning method would you like?") {
            GitCloneMethod::Ssh => format!("git@github.com:{}/{:?}.git", username, &dir.file_name().unwrap().to_owned()),
            GitCloneMethod::Https => format!("https://github.com/{}/{:?}.git", username, &dir.file_name().unwrap().to_owned()), 
        };
        commands_to_run.push(("git", vec!["remote", "set-url", "origin", origin_url.as_str()].into_iter().map(|x| x.to_string()).collect()))
    };

    // Write directories
    let dirs_to_create: Vec<PathBuf> = dirs_to_create
        .into_iter()
        .map(|x| dir.join(x))
        .collect();

    let results: Vec<Result<String, String>> = dirs_to_create
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
    
    // Write files
    for (file, content) in files_to_create {
        match utils::kerblam_create_file(
            &dir.join(file),
            content.as_str(),
            true,
        ) {
            Ok(msg) => println!("{}", msg),
            Err(msg) => {
                println!("{}", msg);
                stop = true
            }
        }
    };

    // Add to gitignore
    match utils::kerblam_create_file(
        &dir.join("./.gitignore"),
        gitignore_content.join("\n").as_str(),
        true
    ) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => {
            println!("{}", msg);
            stop = true;
        }
    }

    if stop {
        return Ok(());
    }
    // Run commands
    for (command, args) in commands_to_run {
        match utils::run_command(command, args.iter().map(|x| &**x).collect()) {
            Ok(_) => (),
            Err(e) => println!("{}", e.msg)
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        Command::New { path } => {
            if let Some(name) = path.file_name() {
                println!("Kerblam! Making new project '{}'", name.to_str().expect("What a weird path!"));
            } else {
                println!("Please provide the name of the project.");
                return Ok(());
            }
            create_kerblam_project(&path)?;
        }
        _ => {
            println!("Not implemented yet!")
        }
    };

    Ok(())
}
