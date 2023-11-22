use serde::Deserialize;
use std::fs;
use std::{collections::HashMap, path::PathBuf};

use crate::errors::StopError;

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

pub fn parse_kerblam_toml(toml_file: &PathBuf) -> Result<KerblamTomlOptions, StopError> {
    let toml_content = match fs::read(toml_file) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(string) => string,
            Err(msg) => Err(StopError {
                msg: msg.to_string(),
            })?,
        },
        Err(reason) => Err(StopError {
            msg: reason.to_string(),
        })?,
    };
    let config: KerblamTomlOptions = match toml::from_str(toml_content.as_str()) {
        Ok(data) => data,
        Err(msg) => Err(StopError {
            msg: msg.to_string(),
        })?,
    };

    Ok(config)
}
