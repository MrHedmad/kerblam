use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

use crate::cache::{check_last_profile, delete_last_profile, get_cache};
use crate::cli::Executable;
use crate::execution::{Executor, FileMover};
use crate::options::extract_profile_paths;
use crate::options::find_and_parse_kerblam_toml;
use crate::options::KerblamTomlOptions;
use crate::options::Pipe;
use crate::utils::find_pipe_by_name;
use crate::utils::print_md;
use crate::utils::update_timestamps;

use anyhow::{anyhow, bail, Result};
use clap::Args;

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

pub fn kerblam_run_project(
    config: KerblamTomlOptions,
    pipe: Pipe,
    runtime_dir: &PathBuf,
    profile: Option<String>,
    ignore_container: bool,
    skip_build_cache: bool,
    extra_args: Option<Vec<String>>,
) -> Result<()> {
    let pipe = if ignore_container {
        pipe.drop_env()
    } else {
        pipe
    };
    log::debug!("Profile: {:?}", profile);

    // Create an executor for later.
    let executor: Executor = pipe.into_executor(runtime_dir)?;

    // Handle renaming the input files if we are in a profile
    let unwinding_paths: Vec<FileMover> = if let Some(profile) = profile.clone() {
        // This should mean that there is a profile with the same name in the
        // config...
        let profile_paths = extract_profile_paths(&config, profile.as_str(), true)?;

        // Check the cache (if there) what the last profile was.
        // If it was this one, we should not update the file creation time
        // as we move them around, or the make pipelines re-run from the
        // beginning even if we did nothing to them
        let cached_run = check_last_profile(profile);
        let cached_run = cached_run.unwrap_or(false);
        log::debug!("Checked cached profile: {}", cached_run);

        // Rename the paths that we found
        let move_results: Vec<Result<FileMover, anyhow::Error>> = profile_paths
            .into_iter()
            .map(|x| x.rename(!cached_run))
            .collect();
        // If they are all ok, return the vec
        if move_results.iter().all(|x| x.is_ok()) {
            move_results.into_iter().map(|x| x.unwrap()).collect()
        } else {
            // Not all is ok, unwind and bail out.
            // I have to clone here as I need to consume the vector twice,
            // but the toplevel vector cannot be cloned, so I clone and then
            // ref deeper in. A bit clunky.
            let unwindable: Vec<FileMover> = move_results
                .iter()
                .filter_map(|x| x.as_ref().ok())
                .map(|x| x.to_owned())
                .collect();
            for mover in unwindable {
                // I don't use the result for the same reason.
                let _ = mover.rename(!cached_run);
            }

            let failed: Vec<anyhow::Error> =
                move_results.into_iter().filter_map(|x| x.err()).collect();

            bail!(
                "Some profiled paths failed to be moved: {}",
                failed
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
    } else {
        // If we are not in a profile now, but we were before, we should
        // re-touch all the old profile paths just to be safe that the
        // whole pipeline is re-run again with the new data
        let last_cache = get_cache();
        if last_cache
            .clone()
            .is_some_and(|x| x.last_executed_profile.is_some())
        {
            log::debug!("Should re-touch profile files.");
            // We can avoid checking if all files exist or in general if the
            // profile is valid, since we just touch the existing files and
            // that is it - we don't enact the profile.
            let profile_paths = extract_profile_paths(
                &config,
                &last_cache.unwrap().last_executed_profile.unwrap(),
                false,
            )
            .unwrap_or_else(|_| {
                log::warn!("Could not find old profile paths. Skipping re-touching.");
                vec![]
            });

            for mover in profile_paths {
                log::debug!("Touching {:?}", &mover.clone().get_from());
                match update_timestamps(&mover.clone().get_from()) {
                    Ok(_) => (),
                    Err(e) => {
                        log::warn!(
                            "Cannot touch {:?}: {:?}. Was the file deleted? Ignoring it.",
                            &mover.clone().get_from(),
                            e
                        )
                    }
                }
            }

            // We are done. We can delete the last profile.
            let _ = delete_last_profile();
        }

        vec![]
    };

    // Build the extra env vars that we want to set during the execution
    let env_vars: HashMap<String, String> = if let Some(profile) = profile {
        HashMap::from([("KERBLAM_PROFILE".to_string(), profile)])
    } else {
        HashMap::new()
    };

    // Execute the executor
    let runtime_result = executor.execute(&config, env_vars, skip_build_cache, extra_args);

    // Undo the input file renaming
    if !unwinding_paths.is_empty() {
        log::info!("Undoing profile...");
        for item in unwinding_paths.into_iter().rev() {
            // If this worked before, it should work now, that is why I discard the
            // result...
            // TODO: This might be a bad idea.
            //
            // We can skip updating timestamps at this stage
            let _ = item.rename(false);
        }
    }

    // Try and destroy the symlinks

    // Return either an error or OK, if the pipeline finished appropriately
    // or crashed and burned.
    if runtime_result.is_ok() {
        match runtime_result.unwrap() {
            Some(res) => {
                if res.success() {
                    Ok(())
                } else {
                    Err(anyhow!("Process exited with error: {res:?}"))
                }
            }
            None => Err(anyhow!("Process killed.")),
        }
    } else {
        Err(anyhow!("Process exited."))
    }
}
