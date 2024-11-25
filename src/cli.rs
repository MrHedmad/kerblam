use clap::{Parser, Subcommand};
use std::ffi::OsString;

use anyhow::Result;

use crate::commands::{
    DataCommand, IgnoreCommand, NewCommand, PackageCommand, ReplayCommand, RunCommand,
};

/// This string is displayed when the help message is invoked.
const KERBLAM_LONG_ABOUT: &str = concat!(
    "  _  __           _     _               _ \n",
    " | |/ / ___  _ _ | |__ | | __ _  _ __  | |\n",
    " | ' < / -_)| '_|| '_ \\| |/ _` || '  \\ |_|\n",
    " |_|\\_\\\\___||_|  |_.__/|_|\\__,_||_|_|_|(_)\n",
    "                                          \n",
    "Manage your data analysis projects quickly and easily.\n\n",
    "Kerblam! is a project manager that lets you focus on getting things done,\n",
    "with it taking care of tedious or tricky parts of project management.\n\n",
    "You can find the Kerblam! manual online at https://kerblam.dev/.\n",
    "The source code is available at https://github.com/MrHedmad/kerblam"
);

/// A very simple trait that every command has
///
/// This makes it easier to implement the right signature with tab-completion
pub trait Executable {
    fn execute(self) -> Result<()>;
}

#[derive(Parser, Debug)]
#[command(author = "hedmad", version, about = KERBLAM_LONG_ABOUT)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

// Each command has its own docstring near the implementation of the
// **Command struct. See the deeper files.

#[derive(Subcommand, Debug)]
enum Command {
    New(NewCommand),
    Run(RunCommand),
    Data(DataCommand),
    Replay(ReplayCommand),
    Package(PackageCommand),
    Ignore(IgnoreCommand),
}

impl Executable for Command {
    fn execute(self) -> Result<()> {
        match self {
            Self::New(x) => x.execute(),
            Self::Run(x) => x.execute(),
            Self::Data(x) => x.execute(),
            Self::Replay(x) => x.execute(),
            Self::Package(x) => x.execute(),
            Self::Ignore(x) => x.execute(),
        }
    }
}

// This is the same as the trait `Executable`, but for module privacy reasons
// it's not implemented as such, but as a normal (associated) function.
// Plus, it's hard to store the args data in the Cli struct, so an associated
// function would be needed anyway.
impl Cli {
    pub fn execute<I, T>(raw_args: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = Cli::parse_from(raw_args);

        args.command.execute()
    }
}
