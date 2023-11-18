use clap::{Parser, Subcommand};
use colored::ColoredString;
use std::{path::PathBuf, error::Error, str::FromStr, io::ErrorKind};
use serde::Deserialize;
use std::fs;

const KERBLAM_LONG_ABOUT: &str = "Remember, if you want it - Kerblam it!";

#[derive(Parser, Debug)]
#[command(author = "hedmad", about = KERBLAM_LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Command   
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new Kerblam! project.
    ///
    /// A Kerblam! project has a kerblam.toml file in its root, and enjoys
    /// some nice perks:
    /// - Allows for easy remote data access, by just specifying URLs and
    ///   access tokens;
    /// - Can package and export data quickly and easily to share the project
    ///   with colleagues;
    /// - Allows to manage and run multiple makefiles for different versions
    ///   of the project;
    /// - Leverages git to isolate, rollback and run the project at a different
    ///   tag;
    /// - Cleans up intermediate and output files quickly;
    /// - Manages Docker environments and runs code in them for you.
    ///
    /// To transform a project to a Kerblam! project just make the kerblam.toml
    /// file yourself.
    New {
        /// The path to initialize. Defaults to current working directory.
        path: Option<PathBuf>
    },
    /// Package a Kerblam! project for later
    ///
    /// This strips out the intermediate data files and creates an archive,
    /// optionally with git
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

fn kerblam_create_dir(dir: &PathBuf) -> String {
    if dir.exists() && dir.is_dir() {
        return format!("🔷 {:?} was already there!", dir);
    }
    if dir.exists() && dir.is_file() {
        return format!("❌ {:?} is a file!", dir);
    }

    match fs::create_dir_all(dir) {
        Ok(_) => format!("✅ {:?} created!", dir),
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => format!("❌ No permission to write {:?}!", dir),
                kind => format!("❌ Failed to write {:?} - {:?}", dir, kind)
            }
        }
    }

}


fn create_kerblam_project(dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    kerblam_create_dir(dir);

    // A kerblam project is made of the kerblam.toml file
    println!("{}", kerblam_create_dir(&dir.join("./data/in")));
    println!("{}", kerblam_create_dir(&dir.join("./data/out")));
    println!("{}", kerblam_create_dir(&dir.join("./src/modules")));

    fs::write(dir.join("./kerblam.toml"), "")?;

    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    
    match args.command {
        Command::New { path } => {
            let path: PathBuf = path.unwrap_or(PathBuf::from_str("")?);
            create_kerblam_project(&path)?;

        },
        _ => {println!("Not implementede yet!")} 
    };

    Ok(())
}

