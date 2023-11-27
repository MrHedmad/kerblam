use serde::Deserialize;
use std::fs;
use std::{collections::HashMap, path::PathBuf};
use std::path::Path;

use anyhow::Result;

// TODO: Remove the #[allow(dead_code)] calls when we actually use the
// options here.

// Note: i keep all the fields that are not used to private until we
// actually support their usage.

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DataOptions {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    intermediate: Option<PathBuf>,
    temporary: Option<PathBuf>,
    // Profiles are like HashMap<profile_name, HashMap<old_file_name, new_file_name>>
    pub profiles: Option<HashMap<String, HashMap<PathBuf, PathBuf>>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CodeOptions {
    root: Option<PathBuf>,
    modules: Option<PathBuf>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
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

