use serde::Deserialize;
use std::path::PathBuf;

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
    modules: Option<PathBuf>,
}

#[derive(Deserialize)]
struct KerblamTomlOptions {
    data: DataOptions,
    code: CodeOptions,
}
