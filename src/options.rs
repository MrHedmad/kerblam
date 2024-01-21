use serde::Deserialize;
use std::env::current_dir;
use std::fmt::Display;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use url::Url;

use crate::commands::run::Executor;
use crate::utils::{find_files, warn_kerblam_version};

// TODO: Remove the #[allow(dead_code)] calls when we actually use the
// options here.

// Note: i keep all the fields that are not used to private until we
// actually support their usage.

// TODO: Consider using serde defaults here instead of some of the options.
//
// TODO: Apparently something called 'lenses' could really help reduce
//      most of this code by allowing access to nested 'Option'-heavy structs
//      but for the love of god I can't get it to work.

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct DataPaths {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    intermediate: Option<PathBuf>,
    temporary: Option<PathBuf>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct DataOptions {
    pub paths: Option<DataPaths>,
    // Profiles are like HashMap<profile_name, HashMap<old_file_name, new_file_name>>
    pub profiles: Option<HashMap<String, HashMap<PathBuf, PathBuf>>>,
    pub remote: Option<HashMap<Url, PathBuf>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct CodeOptions {
    pub env_dir: Option<PathBuf>,
    pub pipes_dir: Option<PathBuf>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Meta {
    pub version: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct KerblamTomlOptions {
    pub meta: Option<Meta>,
    pub data: Option<DataOptions>,
    pub code: Option<CodeOptions>,
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

impl Into<PathBuf> for RemoteFile {
    fn into(self) -> PathBuf {
        self.path
    }
}

impl Into<Url> for RemoteFile {
    fn into(self) -> Url {
        self.url
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
            let header = pieces[0].replace("\n", " ");
            return PipeDescription { header, body: None };
        }

        let header = pieces[0].replace("\n", " ");
        let body: Vec<String> = pieces
            .into_iter()
            .skip(1)
            .map(|x| x.replace("\n", " "))
            .collect();
        let body: String = body.join("\n\n");

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

    pub fn to_executor(
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
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if self.env_path.is_none() {
            "\t"
        } else {
            "\t🐋"
        };

        let desc = self
            .description()
            .expect("Could not parse file to look for description.");

        match desc {
            Some(desc) => {
                write!(f, "{} {} :: {}", prefix, self.name(), desc.header)
            }
            None => {
                write!(f, "{} {}", prefix, self.name())
            }
        }
    }
}

// TODO: These methods are repetitive, verbose and ugly
// There must be better ways to do this, but I'm looking for something
// quick and dirty to get the job done.

impl KerblamTomlOptions {
    /// Return objects representing remote files specified in the config
    pub fn remote_files(&self) -> Vec<RemoteFile> {
        let root_data_dir = self.input_data_dir();
        log::debug!("Remote file save dir is {root_data_dir:?}");

        self.data
            .clone()
            .and_then(|x| x.remote)
            .and_then(|y| {
                Some(
                    y.iter()
                        .map(|pairs| RemoteFile {
                            url: pairs.0.clone(),
                            path: root_data_dir.join(pairs.1),
                        })
                        .collect(),
                )
            })
            .unwrap_or(vec![])
    }

    // Return the path to the input data directory
    pub fn input_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.input)
                .unwrap_or(PathBuf::from("data/in")),
        )
    }
    pub fn output_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.output)
                .unwrap_or(PathBuf::from("data/out")),
        )
    }
    pub fn intermediate_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.intermediate)
                .unwrap_or(PathBuf::from("data")),
        )
    }
    pub fn temporary_data_dir(&self) -> PathBuf {
        current_dir().unwrap().join(
            self.data
                .clone()
                .and_then(|x| x.paths)
                .and_then(|x| x.input)
                .unwrap_or(PathBuf::from("/tmp")),
        )
    }

    fn extract_dir(&self, target: PathBuf) -> Vec<PathBuf> {
        let filter_dirs = vec![
            self.input_data_dir(),
            self.intermediate_data_dir(),
            self.temporary_data_dir(),
            self.output_data_dir(),
        ];

        // Remove the parent filter dirs, otherwise we can't look into here
        let filter_dirs: Vec<PathBuf> = filter_dirs
            .into_iter()
            .filter(|path| !target.starts_with(path))
            .collect();

        log::debug!("Extracting {target:?} with filters {filter_dirs:?}");

        find_files(target, (!filter_dirs.is_empty()).then(|| filter_dirs))
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

    #[allow(dead_code)]
    /// Return all the locally present temporary files
    ///
    /// This filters out output, input and intermediate files from the call
    pub fn temporary_files(&self) -> Vec<PathBuf> {
        self.extract_dir(self.temporary_data_dir())
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
            .map(|x| (x.file_stem().unwrap().to_string_lossy().to_string(), x))
            .collect();
        let envs_names: Vec<(String, PathBuf)> = env_paths
            .into_iter()
            .map(|x| (x.file_stem().unwrap().to_string_lossy().to_string(), x))
            .collect();
        let mut pipes: Vec<Pipe> = vec![];

        // TODO: I duct-taped this loop with inner clone(), but I'm 76% sure
        // that we can do it with references.

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
                    env_path: None,
                })
            }
        }

        pipes
    }

    /// Return all paths to pipes.
    fn pipes_paths(&self) -> Vec<PathBuf> {
        let pipes = self
            .code
            .clone()
            .and_then(|x| x.pipes_dir)
            .unwrap_or_else(|| current_dir().unwrap().join("src/pipes"));

        find_files(pipes, None)
    }

    /// Return all paths to environments.
    fn env_paths(&self) -> Vec<PathBuf> {
        let env = self
            .code
            .clone()
            .and_then(|x| x.env_dir)
            .unwrap_or_else(|| current_dir().unwrap().join("src/dockerfiles"));

        find_files(env, None)
    }
}
