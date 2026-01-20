use std::{env::current_dir, process::exit};

use crate::{
    cache::{get_cache, RunMetadata},
    cli::Executable,
    filesystem_state::generate_tree_summary,
    options::find_and_parse_kerblam_toml,
    utils::{find_pipe_by_name, print_md},
};
use clap::Args;

/// Inspect a workflow to obtain extra information about it
///
/// This displays information regarding a specific workflow, like its title,
/// description, file path, associated runtime container (if any), and more.
///
/// Examples:
///     > See a list of all workflows
///         kerblam inspect
///
///     > See information about a workflow named 'my_workflow'
///         kerblam inspect my_workflow
#[derive(Args, Debug, Clone)]
pub struct InspectCommand {
    /// Name of the workflow to be inspected
    module_name: Option<String>,
    /// Show the latest run information for this workflow
    #[arg(long, short)]
    trace: bool,
}

impl Executable for InspectCommand {
    fn execute(self) -> anyhow::Result<()> {
        let config = find_and_parse_kerblam_toml()?;
        let pipe = find_pipe_by_name(&config, self.module_name.clone())?;
        let here = current_dir().unwrap().to_string_lossy().into_owned();
        // It's safe to unwrape the name, the previous call would have failed.
        let module_name = self.module_name.unwrap();

        let cache = get_cache();
        let last_run_trace: Option<RunMetadata> = if cache.is_none() {
            None
        } else {
            let cache = cache.unwrap();
            if cache.run_metadata.is_none() {
                None
            } else {
                let meta = cache.run_metadata.unwrap();
                let last_run = meta
                    .into_iter()
                    .rev()
                    .find(|s| s.pipe_path == pipe.pipe_path);

                last_run
            }
        };

        // Unpack the description struct and set defaults if needed.
        let desc = match pipe.description() {
            Ok(x) => x,
            Err(reason) => {
                println!(
                    "Failed to read pipe description: {}. Exiting early.",
                    reason
                );
                exit(1);
            }
        };
        let header = match &desc {
            Some(x) => x.header.to_owned(),
            None => "No workflow title found.".into(),
        };

        println!("🔎 Inspecting workflow '{}' 🔎\n", module_name);
        println!("👑 Workflow title: {}", header);
        println!(
            "📁 Path to the workflow file:  {}",
            pipe.pipe_path.display()
        );
        // Here I do a bit of a hack, making the path relative by force.
        // but it should always be fine as relative, so I think I can get away with it.
        println!(
            "🐋 Path to the container file: {}",
            match pipe.env_path {
                Some(path) => format!(".{}", path.to_str().unwrap().strip_prefix(&here).unwrap()),
                None => "❌ No container file found.".into(),
            }
        );
        println!("");
        match &desc {
            Some(content) if content.body.is_some() => print_md(&format!(
                "-- Workflow Description --\n{}",
                content.body.to_owned().unwrap()
            )),
            _ => {
                println!("❌ No workflow description found.")
            }
        }

        if !self.trace {
            if last_run_trace.is_some() {
                println!("Omitted trace information. Use --trace to include.");
            } else {
                println!("No trace information for this run.")
            }
            return Ok(());
        };

        // We have to show the trace information
        println!(
            "-- Last run file differences --\n\n{}",
            generate_tree_summary(&last_run_trace.unwrap().modified_files)
        );

        Ok(())
    }
}
