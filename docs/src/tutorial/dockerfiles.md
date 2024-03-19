# Writing Dockerfiles for Kerblam!

When you write dockerfiles for use with Kerblam! there are a few things you
should keep in mind:
- Kerblam! will automatically set the proper entrypoints for you;
- The build context of the dockerfile will always be the place where the
  `kerblam.toml` file is.
- Kerblam! will not ignore any file for you.
- The behaviour of `kerblam package` is *slightly* different than `kerblam run`,
  in that the context of `kerblam package` is an isolated "restarted" project,
  as if `kerblam data clean --yes` was run on it, while the context of
  `kerblam run` is the current project, as-is.

This means a few things:

### `COPY` directives are executed in the root of the repository
This is exactly what you want, usually.
This makes it possible to copy the whole project over to the container by just
using `COPY . .`.

### The `data` directory is excluded from packages
If you have a `COPY . .` directive in the dockerfile, it will behave differently
when you `kerblam run` versus when you `kerblam package`.

When you run `kerblam package`, Kerblam! will create a temporary build context
with no input data.
This is what you want: Kerblam! needs to separately package your (precious)
input data on the side, and copy in the container only code and other execution-specific
files.

In a run, the current local project directory is used as-is as a build context.
This means that the `data` directory will be copied over.
At the same time, Kerblam! will also *mount* the same directory to the running
container, so the copied files will be "overwritten" by the live mountpoint
while to container is running.

This generally means that copying the whole data directory is useless in a run,
and that it cannot be done during packaging.

Therefore, a best practice is to ignore the contents of the data folders in the
`.dockerignore` file.
This makes no difference while packaging containers but a big difference when
running them, as docker skips copying the useless data files.

To do this in a standard Kerblam! project, simply add this to your `.dockerignore`:
```
# Ignore the intermediate/output directory
data
```

You might also want to add any files that you know are not useful in the docker
environment, such as local python virtual environments.

### Your dockerfiles can be very small
Since the configuration is handled by Kerblam!, the main reason to write dockerfiles
is to install dependencies.

This makes your dockerfiles generally very small:
```dockerfile
FROM ubuntu:latest

RUN apt-get update && apt-get install # a list of packages

COPY . .
```

You might also be interested in the article
'[best practices while writing dockerfiles](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)'
by Docker.

### Docker images are named based on the pipeline name
If you run `kerblam run my_pipeline` twice, the same container is built to run
the pipeline twice, meaning that caching will make your execution quite fast if
you place the `COPY . .` directive near the bottom of the dockerfile.

This way, you can essentially work exclusively in docker and never install
anything locally.

Kerblam! will name the containers for the pipelines as `<pipeline name>_kerblam_runtime`.
