use serde::Deserialize;
use std::env::current_dir;
use std::fs;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use url::Url;

// TODO: Remove the #[allow(dead_code)] calls when we actually use the
// options here.

// Note: i keep all the fields that are not used to private until we
// actually support their usage.

// Serde requires functions like this, for now. See serde-rs/serde/issues/368
fn d_input_dir() -> PathBuf {
    "./data/in".into()
}
fn d_output_dir() -> PathBuf {
    "./data/out".into()
}
fn d_int_dir() -> PathBuf {
    "./data".into()
}
fn d_temp_dir() -> PathBuf {
    "/tmp/data".into()
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct DataPaths {
    #[serde(default = "d_input_dir")]
    input: PathBuf,
    #[serde(default = "d_output_dir")]
    output: PathBuf,
    #[serde(default = "d_int_dir")]
    intermediate: PathBuf,
    #[serde(default = "d_temp_dir")]
    temporary: PathBuf,
}

fn d_pipes_dir() -> PathBuf {
    "./src/pipes/".into()
}
fn d_env_dir() -> PathBuf {
    "./src/dockerfiles".into()
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
    #[serde(default = "d_env_dir")]
    env_dir: PathBuf,
    #[serde(default = "d_pipes_dir")]
    pipes_dir: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct KerblamTomlOptions {
    pub data: Option<DataOptions>,
    code: Option<CodeOptions>,
}

pub fn parse_kerblam_toml(toml_file: impl AsRef<Path>) -> Result<KerblamTomlOptions> {
    let toml_file = toml_file.as_ref();
    log::debug!("Reading {:?} for TOML options...", toml_file);
    let toml_content = String::from_utf8(fs::read(toml_file)?)?;
    let config: KerblamTomlOptions = toml::from_str(toml_content.as_str())?;

    Ok(config)
}

pub struct RemoteFile {
    pub url: Url,
    pub path: PathBuf,
}

impl KerblamTomlOptions {
    pub fn remote_files(&self) -> Vec<RemoteFile> {
        let here = &current_dir().unwrap();

        let root_data_dir = self
            .data
            .clone()
            .and_then(|x| x.paths)
            .and_then(|x| x.input.into())
            .or_else(|| Some(here.join("data/in")))
            .unwrap();
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
}
