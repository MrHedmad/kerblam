use clap::{Parser, Subcommand};
use std::{path::PathBuf, error::Error, str::FromStr, io::{ErrorKind, self, Write}};
use serde::Deserialize;
use std::fs;

const KERBLAM_LONG_ABOUT: &str = "Remember, if you want it - Kerblam it!";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
struct StopError {
    msg: String,
}

#[derive(Parser, Debug)]
#[command(author = "hedmad", about = KERBLAM_LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Command   
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new Kerblam! project.
    New {
        /// Path of the new project.
        path: String
    },
    /// Package a Kerblam! project for later
    Pack {
        /// Where to save the packed project
        path: Option<PathBuf>
    },
    /// Clone a remote git Kerblam! project
    Clone {
        /// The remote git URL to clone
        remote_url: String
    },
    /// Hydrate an existing Kerblam! project
    Hydrate {
        path: Option<PathBuf>
    },
    /// Unpack a packed Kerblam! project
    Unpack {
        packed_path: PathBuf,
        destination: Option<PathBuf>
    },
    /// Run a Kerblam! project
    Run {
        /// Name of the module to run
        module_name: String,
        /// Path to the Kerblam! project to be run
        project_path: Option<PathBuf>
    }
}

#[derive(Deserialize)]
struct DataOptions {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    intermediate: Option<PathBuf>,
    temporary: Option<PathBuf>,
}

#[derive(Deserialize)]
struct CodeOptions {
    root: Option<PathBuf>,
    modules: Option<PathBuf>
}

#[derive(Deserialize)]
struct KerblamTomlOptions{
    data: DataOptions,
    code: CodeOptions
}

/// Create a directory, and prepare an output message.
///
/// Creates a dir only if it does not exist.
/// 
/// Output messages are wrapped in `Ok` or `Err` in order to differentiate
/// between the two. On Err, the program is expected to exit, since the
/// folder does not exist AND it cannot be created.
fn kerblam_create_dir(dir: &PathBuf) -> Result<String, String> {
    if dir.exists() && dir.is_dir() {
        return Ok(format!("🔷 {:?} was already there!", dir));
    }
    if dir.exists() && dir.is_file() {
        return Err(format!("❌ {:?} is a file!", dir));
    }

    match fs::create_dir_all(dir) {
        Ok(_) => Ok(format!("✅ {:?} created!", dir)),
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => Err(format!("❌ No permission to write {:?}!", dir)),
                kind => Err(format!("❌ Failed to write {:?} - {:?}", dir, kind))
            }
        }
    }
}

fn kerblam_create_file(file: &PathBuf, content: &str, overwrite: bool) -> Result<String, String> {
    if file.exists() && ! overwrite {
        return Err(format!("❌ {:?} was already there!", file));
    }

    if file.exists() && file.is_dir() {
        return Err(format!("❌ {:?} is a directory!", file));
    }

    match fs::write(file, content) {
        Ok(_) => Ok(format!("✅ {:?} created!", file)),
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => Err(format!("❌ No permission to write {:?}!", file)),
                kind => Err(format!("❌ Failed to write {:?} - {:?}", file, kind))
            }
        }
    }
}
enum UserResponse {
    Yes,
    No,
    Abort,
    Explain
}

impl UserResponse {
    fn parse(other: String) -> Option<UserResponse> {
        match other.to_lowercase().chars().nth(0) {
            None => None,
            Some(char) => {
                match char {
                    'y' => Some(Self::Yes),
                    'n' => Some(Self::No),
                    'a' => Some(Self::Abort),
                    'e' => Some(Self::Explain),
                    _ => None
                }
            },
        } 
    }

    fn ask_options(prompt: &str, explaination: &str) -> UserResponse {
        let prompt = format!("{} [yes/no/abort/explain]: ", prompt);
        let mut result: Option<UserResponse>;
        loop {
            match ask(&prompt) {
                Err(_) => {
                    println!("Error reading input. Try again?");
                    continue;
                },
                Ok(string) => result = Self::parse(string),
            };
            
            match result {
                None => {
                    println!("Invalid input. Please enter y/n/a/e.");
                    continue;
                },
                Some(Self::Abort) => std::process::abort(),
                Some(Self::Explain) => println!("{}", explaination),
                _ => break
            }
        }

        result.unwrap()
    }
}

impl Into<char> for UserResponse {
    fn into(self) -> char { 
        match self {
            Self::Yes => 'y',
            Self::No => 'n',
            Self::Abort => 'a',
            Self::Explain => 'e',
        }
    }
}

impl Into<bool> for UserResponse {
    fn into(self) -> bool { 
        match self {
            Self::Yes => true,
            Self::No => false,
            Self::Abort => false,
            Self::Explain => false,
        }
    }
}

fn ask(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    Ok(buffer.trim().to_owned())
}

fn clone_repo(owner: &str, name: &str) -> Result<String, String> {

}


fn create_kerblam_project(dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let dirs_to_create: Vec<&str> = vec![
        "", "./data/in", "./data/out", "./src/modules"
    ];
    // Ask for user input
    // I defined `dirs_to_create` before so that if we ever have to add to them
    // dynamically we can do so here.
    let use_python = UserResponse::ask_options("Do you need Python?", concat!(
                "Will you use Python in this project?\n",
                "If so, I'll .gitignore python's files and make a virtual environment."
            ));
    let use_r = UserResponse::ask_options("Do you need R?", concat!(
                "Will you use R in this project?\n",
                "If so, I'll add R to the .gitignore."
            ));
 
    let paths_to_create: Vec<PathBuf> = dirs_to_create.into_iter().map(|x| dir.clone().join(x)).collect();

    let results: Vec<Result<String, String>> = paths_to_create.iter().map(|x| kerblam_create_dir(x)).collect();
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
 
    match kerblam_create_file(&dir.join("./kerblam.toml"), format!("[meta]\nversion: {}", VERSION).as_str(), true) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => {
            println!("{}", msg);
            stop = true
        }
    }

    // This is kind of useless... for now?
    if stop {
        return Ok(())
    }

    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    
    match args.command {
        Command::New { path } => {
            create_kerblam_project(&PathBuf::from_str(path.as_str())?)?;
        },
        _ => {println!("Not implemented yet!")} 
    };

    Ok(())
}

