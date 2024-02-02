use std::env::args;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    kerblam::kerblam(args())?;

    Ok(())
}
