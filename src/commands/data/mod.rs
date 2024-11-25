use crate::cli::Executable;
use crate::options::find_and_parse_kerblam_toml;

use anyhow::Result;
use clap::{Args, Subcommand};

pub mod clean;
pub mod describe;
pub mod fetch;
pub mod pack;

use clean::CleanCommand;
use describe::{DataStatus, FileSize};
use fetch::FetchCommand;
use pack::PackCommand;

/// Show, fetch, delete or manage local data
///
/// You can measure how large a project is on your disk, bulk fetch
/// required data from your project, delete unneeded data or package it
/// up to share it with others.
///
/// If no subcommand is specified, shows the status of current data.
///
/// Examples:
///     > Show the amount of data present locally
///         kerblam data
///
///     > Fetch remote data as specified in kerblam.toml
///         kerblam data fetch
///
///     > Clean all non-essential data for this project
///         kerblam data clean --delete-remote
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct DataCommand {
    #[command(subcommand)]
    subcommand: Option<DataSubcommands>,
}

impl Executable for DataCommand {
    fn execute(self) -> Result<()> {
        match self.subcommand {
            Some(subcommand) => subcommand.execute(),
            None => {
                let config = find_and_parse_kerblam_toml()?;
                let data_info: DataStatus = config.try_into()?;
                println!("{}", data_info);
                Ok(())
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum DataSubcommands {
    Fetch(FetchCommand),
    Clean(CleanCommand),
    Pack(PackCommand),
}

impl Executable for DataSubcommands {
    fn execute(self) -> Result<()> {
        match self {
            // There is no (easy) way around this.
            self::DataSubcommands::Fetch(x) => x.execute(),
            self::DataSubcommands::Clean(x) => x.execute(),
            self::DataSubcommands::Pack(x) => x.execute(),
        }
    }
}
