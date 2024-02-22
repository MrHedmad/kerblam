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

### The `data` directory might be different in packages
If you have a `COPY . .` directive in the dockerfile, it will behave differently
when you `kerblam run` versus when you `kerblam package`.
In a run, **the current, local directory is used as-is as a build context**.
This means that the `data` directory will be copied over.
At the same time, Kerblam! will also *mount* the same directory to the running
container, so the copied files will be "overwritten" by the live mountpoint
while to container is running.

This generally means that copying the whole data directory is useless *unless
you are packaging the pipeline*. 
When you package, you actually do want to copy over your precious files in the
input data directory.

In short, copying the data is useless during `kerblam run`, but useful when you
`kerblam package`.

Therefore, a best practice is to ignore the contents of the intermediate and
temporary data folders in the `.dockerignore` file (which is looked for in
the `src/dockerfiles` folder), but *include* the input files.
This makes no difference while packaging containers but a big difference when
running them, as docker skips copying useless data files.

To do this in a standard Kerblam! project, add this to your `.dockerignore`:
```
# Ignore the intermediate/output directory
data
# Still include input files
!data/in
```

You might also want to add any files that you know are not useful in the docker
environment, such as python virtual environments.

### Your dockerfiles can be very small
Since the configuration is handled by Kerblam!, the main reason to write dockerfiles
is to install dependencies.

This makes your dockerfiles generally very small:
```dockerfile
FROM ubuntu:latest

RUN apt-get update && apt-get install # a list of packages

COPY . .
```

You might also be interested in [best practices while writing dockerfiles](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)
by Docker.


### Docker images are named based on the pipeline name
If you run `kerblam run my_pipeline` twice, the same container is built to run
the pipeline twice, meaning that caching will make your execution quite fast if
you place the `COPY . .` directive near the bottom of the dockerfile.

This way, you can essentially work exclusively in docker and never install
anything locally.

Kerblam! will name the pipelines as `<pipeline name>_kerblam_runtime`.

### Packaged pipelines need your documentation
To re-run packaged pipelines, you generally want to mount the output directory
of the project to a local folder.
This cannot be known by others when you give them a packaged pipeline.
Remember to let them know what the proper internal mountpoint is so that they
can have an easier time replaying your work.