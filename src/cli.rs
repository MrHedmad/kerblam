use clap::{Args, Parser, Subcommand};
use std::env::current_dir;
use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Result;

use crate::commands::FetchCommand;
use crate::commands::{
    clean_data, create_kerblam_project, ignore, kerblam_run_project, package_data_to_archive,
    package_pipe, replay, DataStatus,
};
use crate::options::find_and_parse_kerblam_toml;
use crate::options::ContainerBackend;
use crate::utils::{find_pipe_by_name, print_md};

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

pub trait Executable {
    fn execute(self) -> Result<()>;
}

#[derive(Parser, Debug)]
#[command(author = "hedmad", version, about = KERBLAM_LONG_ABOUT)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Initialize a new, empty Kerblam! project.
///
/// This command asks you a few initialization questions to get you
/// started, such as which programming languages you will use, if you
/// want to use `git` and a remote Git server, etc...
///
/// Examples:
///     > Create a new project in the 'my_project' directory
///          kerblam new ./my_project
///
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct NewCommand {
    /// Path to the new project
    path: PathBuf,
}

impl Executable for NewCommand {
    fn execute(self) -> Result<()> {
        eprintln!("Creating a new project in {:?}!", &self.path);
        create_kerblam_project(&self.path)?;
        Ok(())
    }
}

/// Start a workflow within a Kerblam! project
///
/// Start workflow managers for your workflows, potentially with a data
/// profile attached.
///
/// If no workflow is specified, shows the list of available workflows.
///
/// Examples:
///     > List the available workflows that Kerblam! can manage
///         kerblam run
///
///     > Run the workflow named 'process_csv.sh'
///         kerblam run process_csv
///
///     > Use the 'test' profile with a workflow
///         kerblam run process_csv --profile test
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub struct RunCommand {
    /// Name of the workflow to be started
    module_name: Option<String>,
    /// Name of a data profile to use during this execution
    #[arg(long)]
    profile: Option<String>,
    /// Show the pipe description and exit
    #[arg(long, short, action)]
    desc: bool,
    /// Do not run in container, even if a container is available
    #[arg(long, short, action)]
    local: bool,
    /// Do not use the containerization engine build cache if running in a container
    #[arg(long = "no-build-cache", action)]
    skip_build_cache: bool,
    /// Command line arguments to be passed to child process
    #[clap(last = true, allow_hyphen_values = true)]
    extra_args: Option<Vec<String>>,
}

impl Executable for RunCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        let pipe = find_pipe_by_name(&config, self.module_name)?;
        if self.desc {
            print_md(&pipe.long_description());
            return Ok(());
        }
        kerblam_run_project(
            config,
            pipe,
            &current_dir().unwrap(),
            self.profile,
            self.local,
            self.skip_build_cache,
            self.extra_args,
        )
    }
}

/// Replay a pipeline previously packaged with `package`
///
/// The replay tarball made by `kerblam package` contains all the info
/// to replay a workflow run in the past. This can be done manually by
/// anyone with a containerization engine.
///
/// This command is here for convenience: it unpacks the files for you
/// in their correct positions and starts the replay workflow on top
/// with the correct mountpoints.
///
/// Example:
///     > Replay the 'test.kerblam.tar' replay package
///         kerblam replay test.kerblam.tar
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
struct ReplayCommand {
    /// The name of the compressed replay package
    name: PathBuf,
    /// Where the replay should happen in. Defaults to the current dir
    destination: Option<PathBuf>,
    /// Skip decompressing data? Useful if you replayed before already.
    #[arg(long, short, action)]
    no_decompress: bool,
    /// The name of the container used to run, overriding the
    /// instructions of the .kerblam file.
    #[arg(long, short)]
    tag: Option<String>,
    /// The backend to use, either 'docker' or 'podman'
    #[arg(long, short)]
    #[clap(default_value = "docker")]
    backend: ContainerBackend,
}

impl Executable for ReplayCommand {
    fn execute(self) -> Result<()> {
        replay(
            self.name,
            self.destination
                .unwrap_or(current_dir().expect("Could not find current directory")),
            self.no_decompress,
            self.tag,
            self.backend,
        )?;
        Ok(())
    }
}

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
struct DataCommand {
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

/// Package a workflow for execution later
///
/// Package your workflow in a container primed for execution and
/// a replay tarball with input data and execution parameters.
///
/// If you upload the container to a registry and share your replay
/// tarball, other people can use Kerblam! to re-execute your workflow
/// (or do it manually).
///
/// If you want, you can sign the package by passing the --sign option.
/// This includes your git name and git email in the package.
///
/// Example:
///   kerblam package process_csv --tag username/process_csv:latest
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
struct PackageCommand {
    /// The name of the workflow to package. Must have a related container
    pipe: Option<String>,
    /// The label of the exported container image
    #[arg(long)]
    tag: Option<String>,
    /// If passed, sign the package with git name and email
    #[arg(long)]
    sign: bool,
}

impl Executable for PackageCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        let default_pipe_name = format!("{}_exec", &self.pipe.clone().unwrap_or("x".to_string()));
        let pipe = find_pipe_by_name(&config, self.pipe)?;
        package_pipe(
            config,
            pipe,
            &self.tag.unwrap_or(default_pipe_name),
            self.sign,
        )
    }
}

/// Add paths and whole languages to a .gitignore file
///
/// Supported names can be seen in https://github.com/github/gitignore
/// The input is case-sensitive!
///
/// Examples:
///     > Use the Python gitignore from Github
///         kerblam ignore Python
///
///     > Use the Rust gitignore from Github, and delete duplicates
///         kerblam ignore Rust --compress
///
///     > Ignore a specific file in the project
///         kerblam ignore ./src/test_script.sh
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
struct IgnoreCommand {
    /// The name of a language or a path to ignore
    path_or_name: String,
    /// Should the gitignore be compressed to remove duplicates?
    #[arg(long, action)]
    compress: bool,
}

impl Executable for IgnoreCommand {
    fn execute(self) -> Result<()> {
        // We need this to change the wd correctly
        let _ = find_and_parse_kerblam_toml();
        let gitignore_path = current_dir()?.join(".gitignore");
        ignore(gitignore_path, &self.path_or_name, self.compress)
    }
}

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

/// Clean non-essential data to save disk space
///
/// This removes:
///     - Output data (in the output directory);
///     - Intermediate data (in the intermediate data directory);
/// and all empty directories that are left behind after deletion.
///
/// Examples:
///     > Delete everything that is not precious or remote
///         kerblam data clean
///
///     > Delete everything that is not precious
///         kerblam data clean --include-remote
///
///     > Skip the confirmation prompt
///         kerblam data clean --yes
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
struct CleanCommand {
    #[arg(long, short('r'), action)]
    /// Also delete locally present remote files.
    include_remote: bool,
    #[arg(long, short('d'), action)]
    /// Do not delete locally present directories.
    keep_dirs: bool,
    #[arg(long, short, action)]
    /// Do not ask for any confirmation.
    yes: bool,
    #[arg(long, action)]
    /// Print files that will be cleaned, but don't delete them.
    dry_run: bool,
}

impl Executable for CleanCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        clean_data(
            config,
            !self.include_remote,
            self.keep_dirs,
            self.yes,
            self.dry_run,
        )
    }
}

/// Pack local data for export to others
///
/// This creates a tarball with the local-only input data and the output
/// data currently present on disk.
///
/// The created file will always be a tar.gz file, regardless of the name
/// you give it.
///
/// Example:
///      kerblam data pack my_last_execution.tar.gz
#[derive(Args, Debug, Clone)]
#[command(verbatim_doc_comment)]
struct PackCommand {
    output_path: Option<PathBuf>,
}

impl Executable for PackCommand {
    fn execute(self) -> Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        package_data_to_archive(
            config,
            self.output_path
                .unwrap_or(current_dir()?.join("data/data_export.tar.gz")),
        )
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

impl Cli {
    /// Execute whatever instruction the CLI got
    pub fn execute<I, T>(raw_args: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = Cli::parse_from(raw_args);

        args.command.execute()
    }
}
