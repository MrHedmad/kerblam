use anyhow::*;
use std::env::current_dir;
use std::ffi::OsString;

mod cache;
mod cli;
mod commands;
mod execution;
mod filesystem_state;
mod options;
mod utils;

use cli::Cli;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run Kerblam! with a certain arguments list.
///
/// This is *the* way that Kerblam! is invoked through.
pub fn kerblam<I, T>(arguments: I) -> anyhow::Result<()>
where
    I: Iterator<Item = T>,
    T: Into<OsString> + Clone,
{
    log::debug!("Kerblam! invoked in {:?}", current_dir());

    Cli::execute(arguments)?;

    Ok(())
}
