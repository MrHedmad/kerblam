# Packaging workflows for later

The `kerblam package` command is one of the most useful features of Kerblam!
It allows you to package everything needed to execute a workflow in a docker
container and export it for execution later.

As with `kerblam run`, this is chiefly useful for those times where the
workflow manager of your choice does not support such features, or you do not
wish to use a workflow manager.

You must have a matching dockerfile for every workflow that you want to package,
or Kerblam! won't know what to package your workflow into.

For example, say that you have a `process` pipe that uses `make` to run, and 
requires both a remotely-downloaded `remote.txt` file and a local-only
`precious.txt` file.

If you execute:
```bash
kerblam package process --tag my_process_package
```
Kerblam! will:
- Create a temporary build context;
- Copy all non-data files to the temporary context;
- Build the specified dockerfile as normal, but using this temporary context;
- Create a new `Dockerfile` that:
  - Inherits from the image built before;
  - Copies the Kerblam! executable to the root of the container;
  - Configure the default execution command to something suitable for execution
    (just like `kerblam run` does, but "baked in").
- Build the docker container and tag it with `my_process_package`;
- Export all precious data, the `kerblam.toml` and the `--tag` of the container
  to a `process.kerblam.tar` tarball.

The `--tag` parameter is a docker tag.
You can specify a remote repository with it (e.g. `my_repo/my_container`)
and push it with `docker push ...` (or `podman`) as you would normally do.

> [!TIP]
> If you don't specify a `--tag`, Kerblam! will name the result as `<pipe>_exec`.

### Replaying packaged projects
After Kerblam! packages your project, you can re-run the analysis with
`kerblam replay` by using the `process.kerblam.tar` file:
```bash
kerblam replay process.kerblam.tar ./replay_directory
```
Kerblam! reads the `.kerblam.tar` file, recreates the execution environment from
it by unpacking the packed data, and executes the exported docker container
with the proper mountpoints (as described in the `kerblam.toml` file).

In the container, Kerblam! fetches remote files (i.e. runs `kerblam data fetch`)
and then the workflow is triggered via `kerblam run`.
Since the output folder is attached to the output directory on disk, the 
final output of the workflow is saved locally.

These packages are meant to make workflows reproducible in the long-term.
For day-to-day runs, `kerblam run` is much faster.

> [!IMPORTANT]
> The responsibility of having the resulting docker work in the long-term is
> up to you, not Kerblam!
> For most cases, just having `kerblam run` work is enough for the resulting
> package made by `kerblam package` to work, but depending on your docker
> files this might not be the case.
> Kerblam! does not test the resulting package - it's up to you to do that.
> It's best to try your packaged workflow once before shipping it off.
> 
> However, even a broken `kerblam package` is still useful!
> You can always enter with `--entrypoint bash` and interactively work inside the
> container later, manually fixing any issues that time or wrong setup might
> have introduced.

Kerblam! respects your choices of `execution` options when it packages,
changing backend or working directory as you'd expect.
See [the kerblam.toml specification](../kerblam.toml.html) to learn more.
