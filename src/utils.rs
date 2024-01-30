use anyhow::{anyhow, bail, Result};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use version_compare::Version;
use walkdir::{self, DirEntry};

use crate::options::{KerblamTomlOptions, Pipe};
use crate::VERSION;

/// Create a directory.
///
/// Create a directory and prepare an output message to display to the user.
/// Creates a dir only if it does not exist.
pub fn kerblam_create_dir(dir: impl AsRef<Path>) -> Result<String> {
    let dir = dir.as_ref();

    if dir.exists() && dir.is_dir() {
        return Ok(format!("🔷 {:?} was already there!", dir));
    }
    if dir.exists() && dir.is_file() {
        bail!("❌ {:?} is a file!", dir);
    }

    match fs::create_dir_all(dir) {
        Ok(_) => Ok(format!("✅ {:?} created!", dir)),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(anyhow!("❌ No permission to write {:?}!", dir)),
            kind => Err(anyhow!("❌ Failed to write {:?} - {:?}", dir, kind)),
        },
    }
}

/// Create a file, and prepare an output message.
///
/// Optionally, write content in the file, overwriting it.
pub fn kerblam_create_file(
    file: impl AsRef<Path>,
    content: &str,
    overwrite: bool,
) -> Result<String> {
    let file = file.as_ref();
    if file.exists() && !overwrite {
        bail!("❌ {:?} was already there!", file);
    }

    if file.exists() && file.is_dir() {
        bail!("❌ {:?} is a directory!", file);
    }

    match fs::write(file, content) {
        Ok(_) => Ok(format!("✅ {:?} created!", file)),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(anyhow!("❌ No permission to write {:?}!", file)),
            kind => Err(anyhow!("❌ Failed to write {:?} - {:?}", file, kind)),
        },
    }
}

/// Ask the user for dynamic input, wait for the response, and return it.
///
/// Will trim the input, so if a person just typed '\n' you'd get an empty
/// list.
///
/// TODO: Could potentially overflow if the user types a massive amount of
/// text. But who cares.
pub fn ask(prompt: impl AsRef<str>) -> Result<String> {
    let prompt = prompt.as_ref();
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    Ok(buffer.trim().to_owned())
}

/// To be asked for, you need to be able to show your options.
///
/// TODO: I'd like this to be auto-implemented if you have a
/// From<char>. Maybe pull out all chars?
/// Find a way to iterate over every possible type of the enums?
/// It might be impossible.
pub trait AsQuestion {
    /// Show the options that this object supports when
    /// using `ask_for::<Self>`.
    /// Triggers no issue if it's done wrong, but the user will
    /// be left pondering.
    fn as_options() -> String;
}

pub fn ask_for<T>(prompt: &str) -> T
where
    T: TryFrom<char> + AsQuestion,
{
    loop {
        let t = ask(format!("{} [{}]: ", prompt, T::as_options())).unwrap();
        if t.is_empty() {
            println!("Please choose one option.");
            continue;
        }
        match T::try_from(t.to_ascii_lowercase().chars().next().unwrap()) {
            Ok(value) => return value,
            Err(_) => println!("'{}' is not in {}", t, T::as_options().as_str()),
        }
    }
}

/// A simple Yes or No enum to be asked to the user.
#[derive(Debug, Clone)]
pub enum YesNo {
    Yes,
    No,
}

impl TryFrom<char> for YesNo {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'y' => Ok(Self::Yes),
            'n' => Ok(Self::No),
            _ => Err(anyhow!("Invalid cast value {}!", value))?,
        }
    }
}

impl AsQuestion for YesNo {
    fn as_options() -> String {
        String::from_str("yes/no").unwrap() // This canno fail
    }
}

// This is currently useless, but maybe we could leverage it to not have
// the AsQuestion trait?
impl From<YesNo> for char {
    fn from(val: YesNo) -> Self {
        match val {
            YesNo::Yes => 'y',
            YesNo::No => 'n',
        }
    }
}

impl From<YesNo> for bool {
    fn from(val: YesNo) -> Self {
        match val {
            YesNo::Yes => true,
            YesNo::No => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GitCloneMethod {
    Ssh,
    Https,
}

impl TryFrom<char> for GitCloneMethod {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            's' => Ok(Self::Ssh),
            'h' => Ok(Self::Https),
            _ => Err(anyhow!("Invalid cast value {}!", value)),
        }
    }
}

impl AsQuestion for GitCloneMethod {
    fn as_options() -> String {
        String::from_str("ssh/https").unwrap()
    }
}

pub fn run_command(
    location: Option<impl AsRef<Path>>,
    command: &str,
    args: Vec<&str>,
) -> Result<String> {
    let location = location.map(|path| path.as_ref().to_path_buf());

    log::debug!(
        "Running command in {:?} :: {:?}, {:?}",
        location,
        command,
        args
    );
    print!("🏃 Executing '{} {}'...", command, args.join(" "));

    let output = Command::new(command)
        .current_dir(location.unwrap_or(PathBuf::from_str("./").unwrap()))
        .args(args)
        .output()
        .expect("Failed to spawn process");

    if output.status.success() {
        println!(" Done!");
        Ok(String::from_utf8(output.stdout).expect("Could not parse command output as UTF-8"))
    } else {
        println!();
        Err(anyhow!(
            String::from_utf8(output.stderr).expect("Could not parse command output as UTF-8"),
        ))
    }
}

#[allow(dead_code)]
pub fn clone_repo(
    target: Option<impl AsRef<Path>>,
    repo: &str,
    method: GitCloneMethod,
) -> Result<String> {
    let target = target.map(|path| path.as_ref().to_path_buf());
    let head = match method {
        GitCloneMethod::Ssh => "git@github.com:",
        GitCloneMethod::Https => "https://github.com/",
    };

    let path = match target {
        None => ".".to_string(),
        Some(ref path) => path.to_string_lossy().to_string(),
    };

    run_command(
        target,
        "git",
        vec!["clone", format!("{}{}", head, repo).as_str(), &path],
    )
}

pub fn fetch_gitignore(name: &str) -> Result<String> {
    let url = format!(
        "https://raw.githubusercontent.com/github/gitignore/main/{}.gitignore",
        name
    );

    let response = reqwest::blocking::get(url)?.text()?;
    Ok(response)
}

fn find_path_items_with_filter(
    inspected_path: impl AsRef<Path>,
    top_level_filter: fn(&DirEntry) -> bool,
    filters: Option<Vec<PathBuf>>,
) -> Vec<PathBuf> {
    let inspected_path = inspected_path.as_ref();

    if let Some(filters) = filters {
        // The filters are here to get rid of items that *might* be included
        // by accident, especially when finding data paths.
        //
        // For example, if we want all files in /data/out but we want to
        // preserve the files in /data/, we can add the /data/ filter.
        walkdir::WalkDir::new(inspected_path)
            .into_iter()
            .filter_map(|i| i.ok())
            .filter(|x| {
                // If filter returns true, we return this path
                let mut p = true;
                for path in filters.clone() {
                    if x.path().starts_with(path) {
                        p = false;
                    }
                }
                p
            })
            .filter(top_level_filter)
            .map(|x| x.path().to_owned())
            .collect()
    } else {
        walkdir::WalkDir::new(inspected_path)
            .into_iter()
            .filter_map(|i| i.ok())
            .filter(|path| path.metadata().unwrap().is_file())
            .map(|x| x.path().to_owned())
            .collect()
    }
}

pub fn find_files(inspected_path: impl AsRef<Path>, filters: Option<Vec<PathBuf>>) -> Vec<PathBuf> {
    find_path_items_with_filter(inspected_path, |x| x.metadata().unwrap().is_file(), filters)
}

pub fn find_dirs(inspected_path: impl AsRef<Path>, filters: Option<Vec<PathBuf>>) -> Vec<PathBuf> {
    find_path_items_with_filter(inspected_path, |x| x.metadata().unwrap().is_dir(), filters)
}

pub fn warn_kerblam_version(config: &KerblamTomlOptions) -> () {
    // TODO: is there a way to avoid this clone()? I feel like there should be
    // but I'm not sure.
    let version = config.clone().meta.and_then(|x| x.version);
    let current_ver = Version::from(VERSION);

    let version = match version {
        None => return (),
        Some(ver) => String::from(ver),
    };

    let version = match Version::from(&version) {
        Some(x) => x,
        None => return (),
    };

    let current_ver = match current_ver {
        Some(x) => x,
        None => return (),
    };

    if version != current_ver {
        println!(
            "⚠️  TOML version ({version}) is different from this kerblam version ({current_ver})!",
        )
    };
}

/// Find a pipe by name or die trying
pub fn find_pipe_by_name(config: &KerblamTomlOptions, pipe_name: Option<String>) -> Result<Pipe> {
    let pipes = config.pipes();
    let pipes_list = pipes
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    let pipe_name = match pipe_name {
        None => bail!("No runtime specified. Available runtimes:\n{}", pipes_list),
        Some(name) => name,
    };

    let pipe = {
        let mut hit: Option<Pipe> = None;
        for pipe in pipes {
            if pipe.name() == pipe_name {
                hit = Some(pipe.clone())
            }
        }

        hit
    };

    let pipe = match pipe {
        None => bail!(
            "Cannot find pipe {}. Available runtimes:\n{}",
            pipe_name,
            pipes_list
        ),
        Some(name) => name,
    };

    Ok(pipe)
}
