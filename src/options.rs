use clap::ValueEnum;
use serde::Deserialize;
use std::env::current_dir;
use std::fmt::Display;
use std::fmt::Write;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{self, BufRead};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use url::Url;

use crate::execution::{Executor, FileMover};
use crate::utils::{find_files, get_salt, kerblam_create_dir, push_fragment, warn_kerblam_version};

// Note: i keep all the fields that are not used to private until we
// actually support their usage.

// TODO: Consider using serde defaults here instead of some of the options.
//
// TODO: Apparently something called 'lenses' could really help reduce
//      most of this code by allowing access to nested 'Option'-heavy structs
//      but for the love of god I can't get it to work.

#[derive(Debug, Deserialize, Clone)]
pub struct DataPaths {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    intermediate: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataOptions {
    pub paths: Option<DataPaths>,
    // Profiles are like HashMap<profile_name, HashMap<old_file_name, new_file_name>>
    pub profiles: Option<HashMap<String, HashMap<PathBuf, PathBuf>>>,
    pub remote: Option<HashMap<Url, PathBuf>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CodeOptions {
    pub env_dir: Option<PathBuf>,
    pub pipes_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Meta {
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KerblamTomlOptions {
    pub meta: Option<Meta>,
    pub data: Option<DataOptions>,
    pub code: Option<CodeOptions>,
    #[serde(default)]
    pub execution: ExecutionOptions,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ExecutionOptions {
    #[serde(default)]
    pub backend: ContainerBackend,
    pub workdir: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ContainerBackend {
    Docker,
    Podman,
}

impl Default for ContainerBackend {
    fn default() -> Self {
        Self::Docker
    }
}

impl From<ContainerBackend> for String {
    fn from(val: ContainerBackend) -> Self {
        match val {
            ContainerBackend::Docker => "docker".into(),
            ContainerBackend::Podman => "podman".into(),
        }
    }
}

pub fn parse_kerblam_toml(toml_file: impl AsRef<Path>) -> Result<KerblamTomlOptions> {
    let toml_file = toml_file.as_ref();
    log::debug!("Reading {:?} for TOML options...", toml_file);
    let toml_content = String::from_utf8(fs::read(toml_file)?)?;
    let config: KerblamTomlOptions = toml::from_str(toml_content.as_str())?;

    warn_kerblam_version(&config);

    Ok(config)
}

#[derive(Debug)]
pub struct RemoteFile {
    pub url: Url,
    pub path: PathBuf,
}

impl From<RemoteFile> for PathBuf {
    fn from(val: RemoteFile) -> Self {
        val.path
    }
}

impl From<RemoteFile> for Url {
    fn from(val: RemoteFile) -> Self {
        val.url
    }
}

#[derive(Debug, Clone)]
pub struct Pipe {
    pub pipe_path: PathBuf,
    pub env_path: Option<PathBuf>,
}

pub struct PipeDescription {
    pub header: String,
    pub body: Option<String>,
}

impl PipeDescription {
    fn from_text_box(text: String) -> Self {
        let pieces: Vec<&str> = text.split("\n\n").map(|x| x.trim()).collect();

        if pieces.len() == 1 {
            let header = pieces[0].replace('\n', " ");
            return PipeDescription { header, body: None };
        }

        let header = pieces[0].replace('\n', " ");
        let body: String = pieces[1..].join("\n");

        PipeDescription {
            header,
            body: Some(body),
        }
    }
}

impl Pipe {
    /// Obtain the name of the pipe
    pub fn name(&self) -> String {
        let name = self
            .pipe_path
            .file_stem()
            .expect("Could not extract pipe file name.");

        name.to_string_lossy().into()
    }

    /// Parse the file to obtain the description field
    pub fn description(&self) -> Result<Option<PipeDescription>> {
        let conn = File::open(self.pipe_path.clone())?;

        let lines = io::BufReader::new(conn);

        let mut text_box = String::new();
        for line in lines.lines() {
            let line = line?;
            if line.trim().starts_with("#?") {
                text_box.write_str(&format!("{}\n", line.trim().trim_start_matches("#?")))?;
            }
        }

        if text_box.is_empty() {
            return Ok(None);
        }

        Ok(Some(PipeDescription::from_text_box(text_box)))
    }

    pub fn into_executor(
        self,
        execution_dir: impl AsRef<Path>,
    ) -> std::result::Result<Executor, anyhow::Error> {
        let execution_dir: PathBuf = execution_dir.as_ref().into();
        Executor::create(execution_dir, self.pipe_path, self.env_path)
    }

    /// Drop the environment file from this pipe
    pub fn drop_env(self) -> Self {
        Self {
            pipe_path: self.pipe_path,
            env_path: None,
        }
    }

    /// Generate a long description for this pipe
    pub fn long_description(self) -> String {
        let desc = self
            .description()
            .expect("Could not parse description file");
        let header = format!("{}", self);
        let header = header.trim();

        match desc {
            Some(desc) => match desc.body {
                Some(body) => format!("{}\n{}", header, body),
                None => header.to_string(),
            },
            None => "No description found.".to_string(),
        }
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let container_prefix = if self.env_path.is_none() {
            "◾"
        } else if self.env_path.clone().unwrap().file_stem().unwrap() == "default" {
            "🐟"
        } else {
            "🐋"
        };
        let desc_prefix = if self
            .description()
            .expect("Could not parse description file")
            .is_some_and(|x| x.body.is_some())
        {
            "📜"
        } else {
            "◾"
        };

        let prefix = [container_prefix, desc_prefix].concat();

        let desc = self
            .description()
            .expect("Could not parse file to look for description.");

        match desc {
            Some(desc) => {
                write!(f, "    {} {} :: {}", prefix, self.name(), desc.header)
            }
            None => {
                write!(f, "    {} {}", prefix, self.name())
            }
        }
    }
}

impl KerblamTomlOptions {
    /// Return all paths representing remote files specified in the config
    ///
    /// This includes **all** the files, including those not yet downloaded.
    pub fn remote_files(&self) -> Vec<RemoteFile> {
        let root_data_dir = self.input_data_dir();
        log::debug!("Remote file save dir is {root_data_dir:?}");

        self.data
            .clone()
            .and_then(|x| x.remote)
            .map(|y| {
                y.iter()
                    .map(|pairs| RemoteFile {
                        url: pairs.0.clone(),
                        path: root_data_dir.join(pairs.1),
                    })
                    .collect()
            })
            .unwrap_or_default() // The default vec is just empty
    }

    /// Return the path of the input data directory
    pub fn input_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.input)
                .unwrap_or(PathBuf::from("data/in")),
        )
    }

    /// Return the path of the output data directory
    pub fn output_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.output)
                .unwrap_or(PathBuf::from("data/out")),
        )
    }

    /// Return the path of the intermediate data directory
    pub fn intermediate_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.intermediate)
                .unwrap_or(PathBuf::from("data")),
        )
    }

    /// Extract the content of input/output/intermediate directories
    ///
    /// The `target` is a folder.
    fn extract_dir(&self, target: PathBuf) -> Vec<PathBuf> {
        let filter_dirs = vec![
            self.input_data_dir(),
            self.intermediate_data_dir(),
            self.output_data_dir(),
        ];

        // Remove the parent filter dirs, otherwise we can't look into here
        let filter_dirs: Vec<PathBuf> = filter_dirs
            .into_iter()
            .filter(|path| !target.starts_with(path))
            .collect();

        log::debug!("Extracting {target:?} with filters {filter_dirs:?}");

        find_files(target, (!filter_dirs.is_empty()).then_some(filter_dirs))
    }

    /// Return all the locally present input files
    ///
    /// This filters out output, temporary and intermediate files from the call
    pub fn input_files(&self) -> Vec<PathBuf> {
        self.extract_dir(self.input_data_dir())
    }
    /// Return all the locally present output files
    ///
    /// This filters out input, temporary and intermediate files from the call
    pub fn output_files(&self) -> Vec<PathBuf> {
        self.extract_dir(self.output_data_dir())
    }

    /// Return all the locally present intermediate files
    ///
    /// This filters out output, temporary and input files from the call
    pub fn intermediate_files(&self) -> Vec<PathBuf> {
        self.extract_dir(self.intermediate_data_dir())
    }

    /// Return all locally-present remote files as defined in the config
    pub fn downloaded_files(&self) -> Vec<RemoteFile> {
        let locals = self.input_files();
        self.remote_files()
            .into_iter()
            .filter(|remote| locals.iter().any(|x| x.ends_with(&remote.path)))
            .collect()
    }

    /// Return all remote files that are not found locally
    pub fn undownloaded_files(&self) -> Vec<RemoteFile> {
        let locals = self.input_files();
        self.remote_files()
            .into_iter()
            .filter(|remote| !locals.iter().any(|x| x.ends_with(&remote.path)))
            .collect()
    }

    /// Return all files that are deemed 'volatile'
    ///
    /// Volatile files are output files, remote files and intermediate files.
    pub fn volatile_files(&self) -> Vec<PathBuf> {
        [
            self.intermediate_files(),
            self.output_files(),
            self.downloaded_files()
                .into_iter()
                .map(|x| x.path)
                .collect(),
        ]
        .concat()
        .into_iter()
        // Get rid of hidden files - we ignore them like a good little program should.
        .filter(|x| !x.file_name().unwrap().to_string_lossy().starts_with("."))
        .collect()
    }

    /// Return all files that are deemed 'precious'
    ///
    /// Precious files are input files that cannot be fetched remotely.
    pub fn precious_files(&self) -> Vec<PathBuf> {
        let remote_files = self.remote_files();
        self.input_files()
            .into_iter()
            .filter(|x| !remote_files.iter().any(|y| x == &y.path))
            .collect()
    }

    /// Return all pipes
    pub fn pipes(&self) -> Vec<Pipe> {
        let pipes_paths = self.pipes_paths();
        let env_paths = self.env_paths();

        let pipes_names: Vec<(String, PathBuf)> = pipes_paths
            .into_iter()
            .filter(|x| {
                x.extension()
                    .is_some_and(|x| (x == "makefile") | (x == "sh"))
            })
            .map(|x| (x.file_stem().unwrap().to_string_lossy().to_string(), x))
            .collect();
        let envs_names: Vec<(String, PathBuf)> = env_paths
            .into_iter()
            .filter(|x| x.extension().is_some_and(|x| (x == "dockerfile")))
            .map(|x| (x.file_stem().unwrap().to_string_lossy().to_string(), x))
            .collect();
        let mut pipes: Vec<Pipe> = vec![];

        // TODO: I duct-taped this loop with inner clone(), but I'm 76% sure
        // that we can do it with references.

        let default_dockerfile: Option<PathBuf> = envs_names
            .iter()
            .find(|(name, _)| name == "default")
            .map(|(_, path)| path.to_owned());

        for (pipe_name, pipe_path) in pipes_names {
            let mut found = false;
            for (env_name, env_path) in envs_names.clone() {
                if env_name == pipe_name {
                    pipes.push(Pipe {
                        pipe_path: pipe_path.clone(),
                        env_path: Some(env_path),
                    });
                    found = true;
                }
            }
            if !found {
                pipes.push(Pipe {
                    pipe_path,
                    env_path: default_dockerfile.clone(),
                })
            }
        }

        log::debug!("Found pipes: {pipes:?}");

        pipes
    }

    /// Return all paths to pipes.
    fn pipes_paths(&self) -> Vec<PathBuf> {
        let pipes = self.pipes_dir();
        find_files(pipes, None)
    }

    /// Return the path to the pipes folder
    pub fn pipes_dir(&self) -> PathBuf {
        self.code
            .clone()
            .and_then(|x| x.pipes_dir)
            .unwrap_or_else(|| current_dir().unwrap().join("src/workflows"))
    }

    /// Return the path to the env folder
    pub fn env_dir(&self) -> PathBuf {
        self.code
            .clone()
            .and_then(|x| x.env_dir)
            .unwrap_or_else(|| current_dir().unwrap().join("src/dockerfiles"))
    }

    /// Return all paths to environments.
    fn env_paths(&self) -> Vec<PathBuf> {
        let env = self.env_dir();
        find_files(env, None)
    }
}

fn infer_test_data(paths: Vec<PathBuf>) -> HashMap<PathBuf, PathBuf> {
    let mut matches: HashMap<PathBuf, PathBuf> = HashMap::new();

    for path in paths.clone() {
        let file_name = path.file_name().unwrap().to_string_lossy();
        if file_name.starts_with("test_") {
            let slug = file_name.trim_start_matches("test_");
            let potential_target = path.clone().with_file_name(slug);
            if paths.iter().any(|x| *x == potential_target) {
                matches.insert(potential_target, path);
            }
        }
    }

    matches
}

/// Represents a Kerblam! profile, with a series of files to be moved back and forth
struct Profile<T: Into<PathBuf> + Hash + std::cmp::Eq + Clone + std::fmt::Debug> {
    /// The origin: target list of paths
    ///
    /// 'origin' (the key) is the real file that needs to be substituted,
    /// 'target' (the value) is the file that will substitute the original one
    targets: HashMap<T, Option<T>>,
    /// The root directory where the data lives, e.g. "/../../data/"
    root_dir: PathBuf,
    /// The temporary "swap" directory where the data will be (temporarily) moved
    temp_dir: PathBuf,
}

impl<T: Into<PathBuf> + Hash + std::cmp::Eq + Clone + std::fmt::Debug> Profile<T> {
    #[allow(dead_code)]
    fn add_paths(&mut self, origin: T, target: Option<T>) -> () {
        self.targets.insert(origin, target);
    }

    fn to_filemovers(self) -> Vec<FileMover> {
        log::debug!("Converting hashmap to filemovers: {:?}", self.targets);
        self.targets
            .into_iter()
            .flat_map(|(origin, target)| {
                // The original will always be moved to the temporary
                // This also needs to happen FIRST, not later.
                let original: PathBuf = origin.into();
                let mut res = vec![
                    // We always need to move the original to the temporary file
                    FileMover::from((
                        &self.root_dir.join(&original),
                        push_fragment(
                            &self
                                .temp_dir
                                .join(&original.strip_prefix(&self.root_dir).unwrap()),
                            &format!(".{}", get_salt(5)),
                        ),
                    )),
                ];

                match target {
                    Some(t) => {
                        // This is a regular target: we also need to move the target to the original's
                        // position
                        let target = t.into();
                        res.push(FileMover::from((
                            &self.root_dir.join(&target),
                            &self.root_dir.join(&original),
                        )));
                    }
                    // If there is no target, we don't need to do anything.
                    None => (),
                }

                res
            })
            .collect()
    }

    fn from(data: HashMap<T, T>, root_dir: PathBuf, temp_dir: PathBuf) -> Self {
        let targets = data.into_iter().map(|(origin, target)| {
            // Why do we need to clone here? I don't really get it...
            let target = if target.clone().into().into_os_string() == "_" {
                None
            } else {
                Some(target)
            };
            (origin, target)
        });
        let targets = HashMap::from_iter(targets);

        Self {
            targets,
            root_dir,
            temp_dir,
        }
    }
}

// TODO: This checks for the existence of profile paths here. This is a bad
// thing. It's best to handle the error when we actually do the move.
// This was done this way because I want a nice error list.
// The 'check_existence' check was added to overcome this, but it's a hack.
pub fn extract_profile_paths(
    config: &KerblamTomlOptions,
    profile_name: &str,
    check_existance: bool,
) -> Result<Vec<FileMover>> {
    let root_dir = config.input_data_dir();
    let temp_dir = current_dir().unwrap().join(".kerblam/scratch");

    kerblam_create_dir(&temp_dir)?;

    // Check if the scratch directory is empty. If not, we bail early.
    // If this is not empty, something has gone wrong in a previous run.
    if temp_dir.read_dir()?.next().is_some() {
        bail!(
            "The Kerblam scratch folder ({}) is not empty! Aborting before we do anything stupid.",
            temp_dir.to_string_lossy()
        )
    }

    // If there are no profiles, an empty hashmap is OK intead:
    // we can add the default "test" profile anyway.
    let mut profiles = {
        let data = config.clone().data;
        match data {
            Some(x) => x.profiles.unwrap_or(HashMap::new()),
            None => HashMap::new(),
        }
    };

    // add the default 'test' profile
    if !profiles.keys().any(|x| x == "test") {
        let input_files = config.input_files();
        let inferred_test = infer_test_data(input_files);
        if !inferred_test.is_empty() {
            log::debug!("Inserted inferred test profile: {inferred_test:?}");
            profiles.insert("test".to_string(), inferred_test);
        }
    }

    let profile = profiles
        .get(profile_name)
        .ok_or(anyhow!("Could not find {} profile", profile_name))?;

    let profile = Profile::from(profile.to_owned(), root_dir.clone(), temp_dir);

    let file_movers = profile.to_filemovers();
    log::debug!("Obtained filemovers: {:?}", file_movers);

    // Check if the sources exist, otherwise we crash now, and not later
    // when we actually move the files.
    let origin_exist_check: Vec<String> = file_movers
        .iter()
        .filter_map(|mover| {
            let origin = mover.get_from();
            if origin.exists() {
                // This exists, no error has to be raised.
                None
            } else {
                // This does NOT exist - return the name so we can error.
                Some(origin)
            }
        })
        .map(|x| match x.strip_prefix(&root_dir) {
            Ok(stripped) => stripped.to_path_buf(),
            Err(_) => x,
        })
        .map(|x| format!("\t- {}", x.to_string_lossy()))
        .collect();

    if check_existance {
        if !origin_exist_check.is_empty() {
            bail!(
                "Failed to find some profiles files:\n{}",
                origin_exist_check.join("\n")
            )
        }
    }

    Ok(file_movers)
}
