# Containerized Execution of Pipelines
Kerblam! is primarely useful to ergonomically run pipelines inside containers.

If Kerblam! finds a container recipe (such as a Dockerfile) of the same name
as one of your pipes in the `./src/dockerfiles/` folder
(e.g. `./src/dockerfiles/process_csv.dockerfile`), it will use it automatically
when you execute a pipeline (e.g. `kerblam run process_csv`).

Specifically, it will do something similar to this:
- Copy the pipeline to the root of the directory (as it does normally when you
  launch `kerblam run`), as `./executor`;
- Run `docker build -f ./src/dockerfiles/process_csv.dockerfile --tag process_csv_kerblam_runtime .` to build the container;
- Run `docker run --rm -it -v ./data:/data --entrypoint make process_csv_kerblam_runtime -f /executor`.

This last command runs the container, telling it to execute `make` with
target file `-f /executor`.
Note that it's not *exactly* what kerblam does - it has additional features
to correctly mount your paths, capture `stdin` and `stdout`, etc...

If you have your docker container `COPY . .`, you can then effectively have
Kerblam! run your projects in docker environments, so you can tweak your
dependencies and tooling (which might be different than your dev environment)
and execute in a protected, reproducible environment.

Kerblam! will build the container images without moving the recipies around.
(this is what the `-f` flag does).
The `.dockerfile` in the build context (next to the `kerblam.toml`) is shared
by all pipes.
See the ['using a dockerignore' section](https://docs.docker.com/engine/reference/commandline/build/#use-a-dockerignore-file)
of the Docker documentation for more.
 
You can write dockerfiles for both `make` and `sh` pipes.
Kerblam! configures automatically the correct entrypoint and arguments to run
the pipe in the container.

Read the ["writing dockerfiles for Kerblam!"](dockerfiles.html) section to learn
more about how to write dockerfiles that work nicely with Kerblam! (spoiler: it's
easier than writing canonical dockerfiles!).

For example, you can have the following Dockerfile:
```dockerfile
# ./src/dockerfiles/process_csv.dockerfile

FROM ubuntu:latest

RUN apt-get install python, python-pip && \
    pip install pandas

COPY . .
```
and this dockerignore file:
```dockerfile
# ./src/dockerfiles/.dockerignore
.git
data
venv
```
and simply run `kerblam run process_csv` to build the container and run
your code inside it.

If you run `kerblam run` without a pipeline (or with the wrong pipeline), you
will get the list of available pipelines.
You can see at a glance what pipelines have an associated dockerfile as they
are prepended with a little whale (🐋):
```
Error: No runtime specified. Available runtimes:
    🐋◾ my_pipeline :: Generate the output data in a docker container
    ◾◾ local_pipeline :: Run some code locally
```

### Default dockerfile
Kerblam! will look for a `default.dockerfile` if it cannot find a container
recipe for the specific pipe (e.g. `pipe.dockerfile`), and use that instead.
You can use this to write a generalistic dockerfile that works for your
most simple pipelines.
The whale (🐋) emoji in the list of pipes will be replaced by a fish (🐟) for
pipes that use the default container, so you can identify them easily:
```
Error: No runtime specified. Available runtimes:
    🐋◾ my_pipeline :: Generate the output data in a docker container
    🐟◾ another :: Run in the default container
```

### Switching backends
Kerblam! runs containers by default with Docker, but you can tell it to use
[Podman](https://podman.io/) instead by setting the `execution > backend`
option in your `kerblam.toml`:
```toml
[execution]
backend = "podman" # by default "docker"
```

Podman is slightly harder to set up but has a few benefits, mainly not having
to run in root mode, and being a FOSS program.
For 90% of usecases, you can use `podman` instead of `docker` and it will 
work exactly the same.
Podman and Docker images are interchangeable, so you can use Podman with
dockerhub with no issues.

### Setting the container working directory
Kerblam! does not parse your dockerfile or add any magic to the calls that it
makes based on heuristics.
This means that if you wish to save your code not in the root of the container,
you must tell kerblam! about it.

For instance, this recipe copies the contents of the analysis in a folder
called "`/app`":
```dockerfile
COPY . /app/
```
This one does the same by using the `WORKDIR` directive:
```dockerfile
WORKDIR /app
COPY . .
```
If you change the working directory, let Kerblam! know by setting the
`execution > workdir` option in `kerblam.toml`:
```toml
[execution]
workdir = "/app"
```
In this way, Kerblam! will run the containers with the proper paths.
**This option applies to *ALL* containers managed by Kerblam!**

There is currently no way to configure a different working directory for every
specific dockerfile.
