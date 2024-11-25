use anyhow::{anyhow, bail, Result};
use filetime::{set_file_mtime, FileTime};
use flate2::Compression;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::fs::{self, create_dir_all};
use std::io::{self, ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use termimad::{minimad, MadSkin};

use rand::distributions::{Alphanumeric, DistString};
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

/// A small wrapper to run a certain shell command with some args
///
/// Provides none of the lower-level control of `run_protected_command`.
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
        .output()?;

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

pub fn warn_kerblam_version(config: &KerblamTomlOptions) {
    // TODO: is there a way to avoid this clone()? I feel like there should be
    // but I'm not sure.
    let version = config.clone().meta.and_then(|x| x.version);
    let current_ver = Version::from(VERSION);

    let version = match version {
        None => return,
        Some(ver) => ver,
    };

    let version = match Version::from(&version) {
        Some(x) => x,
        None => return,
    };

    let current_ver = match current_ver {
        Some(x) => x,
        None => return,
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
    let mut pipes_list = pipes.iter().map(|x| x.to_string()).collect::<Vec<String>>();

    pipes_list.sort_unstable();
    // The sorting starts with the emojis, so we sort in the opposite
    // way to show the non-missing (e.g. not "◾") emojis to the top of the
    // list. These are generally the most "interesting" pipelines.
    pipes_list.reverse();

    let pipes_list = pipes_list.join("\n");
    let profiles: Option<Vec<String>> = config
        .clone()
        .data
        .and_then(|x| x.profiles)
        .map(|x| x.into_keys().collect());
    let profiles_list = match profiles {
        None => "No profiles defined".to_string(),
        Some(list) => list.join(", "),
    };

    let pipe_name = match pipe_name {
        None => bail!(
            "No runtime specified. Available runtimes:\n{}\nAvailable profiles: {}.",
            pipes_list,
            profiles_list
        ),
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

// This is stolen from Cargo
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

fn get_termimad_skin() -> MadSkin {
    let mut skin = termimad::MadSkin::default_dark();
    skin.set_headers_fg(termimad::crossterm::style::Color::Yellow);

    skin
}

pub fn print_md(s: &str) {
    let options = minimad::Options::default()
        .clean_indentations(true)
        .continue_spans(true);
    print_text(minimad::parse_text(s, options));
}

fn print_text(text: minimad::Text) {
    let skin = get_termimad_skin();
    let fmt_text = termimad::FmtText::from_text(&skin, text, None);
    println!("{fmt_text}");
}

/// Returns a path with a new dotted extension component appended to the end.
/// Stolen from stackoverflow
pub fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(ext.as_ref());
    os_string.into()
}

/// Create a tarball from a series of files.
///
/// The `.tar` extension in added automatically to `target`.
pub fn tar_files(files: Vec<PathBuf>, strip: impl AsRef<Path>, target: PathBuf) -> Result<PathBuf> {
    let strip = strip.as_ref();

    let data_tar = append_ext("tar", target);
    let data_conn = File::create(&data_tar)?;

    let mut data_tarball = tar::Builder::new(data_conn);

    for item in files {
        let inner = item.strip_prefix(strip)?;
        log::debug!("Adding {item:?} as {inner:?} to {data_tar:?}...");
        data_tarball.append_path_with_name(&item, inner)?;
    }

    data_tarball.finish()?;
    log::debug!("Finished creating tarball {data_tar:?}");

    Ok(data_tar)
}

pub fn gzip_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<PathBuf> {
    let input: PathBuf = input.as_ref().to_path_buf();
    let output: PathBuf = output.as_ref().to_path_buf();

    create_dir_all(output.parent().unwrap())?;

    let mut input = File::open(input)?;

    let zipped = append_ext("gz", output);
    let conn = File::create(&zipped)?;
    let mut encoder = flate2::write::GzEncoder::new(conn, Compression::default());
    std::io::copy(&mut input, &mut encoder)?;

    Ok(zipped)
}

pub fn gunzip_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<PathBuf> {
    let input: PathBuf = input.as_ref().to_path_buf();
    let output: PathBuf = output.as_ref().to_path_buf();

    create_dir_all(output.parent().unwrap())?;

    let input = File::open(input)?;

    let unzipped = output.with_extension("");
    let mut conn = File::create(&unzipped)?;
    let mut decoder = flate2::read::GzDecoder::new(input);

    std::io::copy(&mut decoder, &mut conn)?;

    Ok(unzipped)
}

pub fn update_timestamps(path: &PathBuf) -> anyhow::Result<()> {
    let mut files_touched = 0;
    if path.is_file() {
        files_touched += 1;
        set_file_mtime(path, FileTime::now())?
    } else if path.is_dir() {
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|x| x.ok())
        {
            files_touched += 1;
            if entry.path().is_file() {
                set_file_mtime(entry.path(), FileTime::now())?;
            }
        }
    }
    log::debug!("Re-touched {files_touched} files.");
    Ok(())
}

/// Push a bit of a string to the end of this path
///
/// Useful if you want to add an extension to the path.
/// Requires a clone.
#[allow(dead_code)]
pub fn push_fragment(buffer: impl AsRef<Path>, ext: &str) -> PathBuf {
    let buffer = buffer.as_ref();
    let mut path = buffer.as_os_str().to_owned();
    path.push(ext);
    path.into()
}

/// Get a random alphanumerical string some characters long
#[allow(dead_code)]
pub fn get_salt(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), length)
}
