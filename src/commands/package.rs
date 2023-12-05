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
    // We have to setup the directory to be ready to be executed

    // We now build the docker container as normal...

    // We can already unwind the modifications.

    // We now start from this new docker and add our own layers, copying the
    // precious files and more from the - not ignored - context.
    // We must work in a temporary directory, however.

    // Build this new container and tag it...

    todo!()
}
