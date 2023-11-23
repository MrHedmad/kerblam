use crate::errors::StopError;
use log;
use reqwest;
use std::error::Error;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

/// Create a directory, and prepare an output message.
///
/// Creates a dir only if it does not exist.
///
/// Output messages are wrapped in `Ok` or `Err` in order to differentiate
/// between the two. On Err, the program is expected to exit, since the
/// folder does not exist AND it cannot be created.
pub fn kerblam_create_dir(dir: impl AsRef<Path>) -> Result<String, String> {
    let dir = dir.as_ref();

    if dir.exists() && dir.is_dir() {
        return Ok(format!("🔷 {:?} was already there!", dir));
    }
    if dir.exists() && dir.is_file() {
        return Err(format!("❌ {:?} is a file!", dir));
    }

    match fs::create_dir_all(dir) {
        Ok(_) => Ok(format!("✅ {:?} created!", dir)),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(format!("❌ No permission to write {:?}!", dir)),
            kind => Err(format!("❌ Failed to write {:?} - {:?}", dir, kind)),
        },
    }
}

/// Create a file, and prepare an output message.
///
/// Optionally, write content in the file and overwrite it.
///
/// Output messages are wrapped in `Ok` or `Err` in order to differentiate
/// between the two. On Err, the program is expected to exit, since the
/// folder does not exist AND it cannot be created.
pub fn kerblam_create_file(
    file: impl AsRef<Path>,
    content: &str,
    overwrite: bool,
) -> Result<String, String> {
    let file = file.as_ref();

    if file.exists() && !overwrite {
        return Err(format!("❌ {:?} was already there!", file));
    }

    if file.exists() && file.is_dir() {
        return Err(format!("❌ {:?} is a directory!", file));
    }

    match fs::write(file, content) {
        Ok(_) => Ok(format!("✅ {:?} created!", file)),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(format!("❌ No permission to write {:?}!", file)),
            kind => Err(format!("❌ Failed to write {:?} - {:?}", file, kind)),
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
pub fn ask(prompt: impl AsRef<str>) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt.as_ref());
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
        match T::try_from(t.to_ascii_lowercase().chars().nth(0).unwrap()) {
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
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'y' => Ok(Self::Yes),
            'n' => Ok(Self::No),
            _ => Err("Invalid String!"),
        }
    }
}

impl AsQuestion for YesNo {
    fn as_options() -> String {
        String::from_str("yes/no").unwrap()
    }
}

// This is currently useless, but maybe we could leverage it to not have
// the AsQuestion trait?
impl Into<char> for YesNo {
    fn into(self) -> char {
        match self {
            Self::Yes => 'y',
            Self::No => 'n',
        }
    }
}

impl Into<bool> for YesNo {
    fn into(self) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GitCloneMethod {
    Ssh,
    Https,
}

impl TryFrom<char> for GitCloneMethod {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            's' => Ok(Self::Ssh),
            'h' => Ok(Self::Https),
            _ => Err("Invalid String!"),
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
) -> Result<String, StopError> {
    let location = location.map(|path| path.as_ref().to_path_buf());
    log::debug!(
        "Running command in {:?} :: {:?}, {:?}",
        location,
        command,
        args
    );
    print!(
        "🏃 Executing '{}'...",
        format!("{} {}", command, args.join(" "))
    );

    let output = Command::new(command)
        .current_dir(location.unwrap_or(PathBuf::from_str("./").unwrap()))
        .args(args)
        .output()
        .expect("Failed to spawn process");

    if output.status.success() {
        println!(" Done!");
        return Ok(
            String::from_utf8(output.stdout).expect("Could not parse command output as UTF-8")
        );
    } else {
        println!("");
        return Err(StopError {
            msg: String::from_utf8(output.stderr).expect("Could not parse command output as UTF-8"),
        });
    }
}

#[allow(dead_code)]
pub fn clone_repo(
    target: Option<impl AsRef<Path>>,
    repo: &str,
    method: GitCloneMethod,
) -> Result<String, StopError> {
    let head = match method {
        GitCloneMethod::Ssh => "git@github.com:",
        GitCloneMethod::Https => "https://github.com/",
    };
    let target = target.map(|path| path.as_ref().to_path_buf());

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

pub fn fetch_gitignore(name: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/github/gitignore/main/{}.gitignore",
        name
    );

    let response = reqwest::blocking::get(url)?.text()?;
    Ok(response)
}
