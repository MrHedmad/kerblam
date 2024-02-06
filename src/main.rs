use std::env::args;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = args();

    kerblam::kerblam(args.into_iter())?;

    Ok(())
}
