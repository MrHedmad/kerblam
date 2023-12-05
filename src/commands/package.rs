use crate::options::KerblamTomlOptions;

use anyhow::Result;

/// Package a pipeline for execution later
///
/// # Arguments
///
/// - `config`: The kerblam config for this execution.
/// - `pipe`: The name of the pipe to execute
/// - `package_name`: The name of the docker image built by this execution.
pub fn package_pipe(config: KerblamTomlOptions, pipe: &str, package_name: &str) -> Result<()> {
    todo!()
}
