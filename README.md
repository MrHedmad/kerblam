![If you want it, Kerblam it!](docs/images/logo.png)

> :warning: **Most of this is not implemented yet.**
> Consider this README a roadmap of sort of what kerblam! wants to be.
>
> ```
>            new      run  data clone   package   ignore  link tests
>              |        |     |     |         |        |     |     |
> [progress]>###############----------------------------------------<
> ```

> :warning: `kerblam run` is complete but still untested. Please do use it,
> but be careful. Report any problems in the [issues](https://github.com/MrHedmad/kerblam).
> Thank you kindly!

Kerblam! is a tool that can help you manage data analysis projects.

A Kerblam! project has a `kerblam.toml` file in its root.
Kerblam! allows you to:
- Access remote data quickly, by just specifying URLs to fetch from;
- Package and export data in order to share the project with colleagues;
- Manage and run multiple makefiles for different tasks;
- Leverage git to isolate, rollback and run the project at a different point in time;
- Clean up intermediate and output files quickly;
- Manage Docker environments and run code in them for you.
- Manage the content of you `.gitignore` for you, allowing to add files, 
  directories and even whole languages in one command.
- Make it easy to use `pre-commit` by managing `.pre-commit-hooks`.
- Specify test data to run and quickly use it instead of real data.

To transform a project to a Kerblam! project just make the kerblam.toml
file yourself. To learn how, look at the section below.

# Overview

> :warning: **Note**: In this early stage, commands with :white_check_mark: are
> (mostly) implemented, :construction: are being implemented now, and
> :pushpin: are planned.
>
> More info for implemented commands and commands under implementation are
> available below. Features missing from implemented commands are issues,
> so look there to see what's still missing.
>
> Please imagine that everything is tentative until version `1.0.0`.

- :white_check_mark: `kerblam new` can be used to create a new kerblam!
  project. Kerblam! asks you if you want to use some common programming
  languages and sets up a proper `.gitignore` and pre-commit hooks for you.
- :pushpin: `kerblam clone` can be used to clone a `kerblam` project.
  Kerblam! will ask you to fetch input files, create virtual environments and
  more upon creation.
- :construction: `kerblam data` fetches remote data and saves it locally, manages
  local data and can clean it up.
- :pushpin: `kerblam package` packages your pipeline and exports a `docker`
  image for execution later.
  It's useful for reproducibility purposes as the docker image is primed
  for execution, bundling the kerblam! executable, execution files and non-remote
  data in the blob itself.
- :white_check_mark: `kerblam run` executes the analysis for you,
  by choosing your `makefile`s and `dockerfiles` appropriately and 
  building docker containers as needed.
  Optionally, allows test data or alternative data to be used instead of
  real data, in order to test your pipelines.
- :pushpin: `kerblam ignore` can edit your `.gitignore` file by adding files,
  folders and GitHub's recommended ignores for specific languages in just one command.
- :pushpin: `kerblam link` can be used to move your `data` folder in some other place,
  and leave in its way a symlink, so that everything works just like before.
  This can be useful when your data is particularly bulky and you want to
  save it on some other drive.
- :pushpin: `kerblam data` can be used to check the number and size of local
  data files, and remove/export them.
  Can also be used to just export the output data that the pipeline produces
  for sharing with others or for usage in writing reports/papers.

Kerblam! is *not* and does not want to be:
- A pipeline manager like `snakemake` and `nextflow`: It supports and helps
  you execute `make`, but it does not interfere from then on;
- A replacement for any of the tools it leverages (e.g. `git`, `docker`,
  `pre-commit`);

## Opinions
Kerblam! projects are opinionated:
- The folder structure of your project adheres to the Kerblam! standard,
  although you may configure it in `kerblam.toml`.
  Read about it below.
- You use `make` or bash scripts as your pipeline manager.
- You use `docker` as your virtualisation service.
- You use `git` as your version control system.
  Additionally, you create tags with `git` to record important previous 
  versions of your project.
- You execute your pipelines in a Docker container, and not in your development
  environment.

If you don't like this setup, Kerblam! is not for you.

### Folder structure
Kerblam!, by default, requires the following folder structure (relative to the
root of the project, `./`):
- `./kerblam.toml`: This file contains the options for Kerblam!. It is usually empty.
- `./data/`: This is a directory for the data. Intermediate data files are held here.
- `./data/in/`: Input data files are saved and should be looked for, in here.
- `./data/out/`: Output data files are saved and should be looked for, in here.
- `./src/`: Code you want to be executed should be saved here.
- `./src/pipes/`: Makefiles and bash build scripts should be saved here.
  They have to be written as if they were saved in `./`.
- `./src/dockerfiles/`: Dockerfiles should be saved here. 
- `./env/`: Is the folder where the (eventual) python environment is located.
- `./requirements.txt`: Is the requirements file needed by `pip`;
- `./requirements-dev.txt`: Is the requirements file for development tools 
  needed by `pip`;
  - Optionally, Kerblam! looks for a `pyproject.toml` file and - upon cloning -
    installs the Python package with `pip install -e .`.

You can configure all of these paths in `kerblam.toml`, if you so desire.
This is mostly done for compatibility reasons.

## Contributing
Kerblam! is currently not accepting pull requests as it's still in its infancy.

> :warning: When Kerblam! reaches minimal viability, I'll open PRs.
> You are still welcome to open issues to discuss code quality / structure
> and the design of the tool.

## Licensing and citation
Kerblam! is licensed under the [MIT License](https://github.com/MrHedmad/kerblam/blob/main/LICENSE).

If you wish to cite Kerblam!, please provide a link to this repository.

## Naming
This project is named after the fictitious online shop/delivery company in
[S11E07](https://en.wikipedia.org/wiki/Kerblam!) of Doctor Who.
Kerblam! might be referred to as Kerblam!, Kerblam or Kerb!am, interchangeably,
although Kerblam! is preferred.
The Kerblam! logo is written in the [Kwark Font](https://www.1001fonts.com/kwark-font.html) by [tup wanders](https://www.1001fonts.com/users/tup/).

## Installation
You can find and download a Kerblam! binary in
[the releases tab](https://github.com/mrhedmad/kerblam/releases).
Download it and drop it somewhere that you `$PATH` points to.

If you want to install from source, install Rust and `cargo`, then run:
```bash
cargo install --git https://github.com/MrHedmad/kerblam.git
```

### Requirements
Kerblam! requires a Linux (or generally unix-like) OS.
It also uses binaries that it assumes are already installed:
- GNU `make`: https://www.gnu.org/software/make/
- `git`: https://git-scm.com/
- Docker (as `docker`): https://www.docker.com/

If you can use `git`, `make` and `docker` from your CLI, you should be good.

# Tutorial
This section outlines making and working with a Kerblam! project.
It covers creation, execution and day-to-day tasks that Kerblam! can help with.

## Creating a new project - `kerblam new`
Go in a directory where you want to store the new project and run `kerblam new test-project`.
Kerblam! asks you some setup questions:
- If you want to use [Python](https://www.python.org/);
- If you want to use [R](https://www.r-project.org/);
- If you want to use [pre-commit](https://pre-commit.com/);
- If you have a Github account, and would like to setup the `origin` of your
  repository to Github.com.

Say 'yes' to all of these questions to follow along.
Kerblam! will:
- Make a new git repository,
- create the `kerblam.toml` file,
- create all the directories detailed above,
- make a `.pre-commit-config` file for you,
- create a `venv` environment, as well as the `requirements.txt` and `requirements-dev.txt`
  files,
- and setup the `.gitignore` file with appropriate ignores;

> :warning: Note that Kerblam! will not do an `Initial commit` for you!

You can now start writing code!
The rest of this tutorial outlines common tasks with which you can use `kerblam` for.

## Executing code - `kerblam run`
Kerblam can be used to manage how your project is executed, where and on
what input files.

Say that you have a script in `./src/calc_sum.py`. It takes an input `.csv` file,
processes it, and outputs a new `.csv` file, using `stdin` and `stdout`.

You have an `input.csv` file that you'd like to process with `calc_sum.py`.
You could write a shell script or a makefile with the command to run.
We'll refer to these scripts as "pipes".
Here's an example makefile pipe:

```makefile
./data/out/output.csv: ./data/in/input.csv ./src/calc_sum.py
    cat $< | ./src/calc_sum.py > $@
```

You'd generally place this file in the root of the repository and run `make`
to execute it. This is perfectly fine for projects with a relatively simple
structure and just one execution pipeline.

Imagine however that you have to change your pipeline to run two different
jobs which share a lot of code and input data but have slightly (or dramatically)
different execution.
You might modify your pipe to accept `if` statements, or perhaps write many of
them and run them separately.
In any case, having a single file that has the job of running all the different
pipelines is hard, adds complexity and makes managing the different execution
scripts harder than it needs to be.

Kerblam! manages your pipes for you.
You can write different makefiles and/or shell files for different types of
runs of your project and save them in `./src/pipes/`.
When you `kerblam run`, Kerblam! looks into that folder, finds (by name) the
makefiles that you've written, and brings them to the top level of the project
(e.g. `./`) for execution.

For instance, you could have written a `./src/pipes/process_csv.makefile` for
the previous step, and you could invoke it with `kerblam run process_csv`.
You could then write more makefiles or shell files for other tasks and run
them similarly.

Kerblam! looks for files ending in the `.makefile` extension for makefiles and 
`.sh` for shell files.

### Containerized execution
If Kerblam! finds a Dockerfile of the same name as one of your pipes in the
`./src/dockerfiles/` folder (e.g. `./src/dockerfiles/process_csv.dockerfile`),
it will:
- Copy the dockerfile to the top folder, as `Dockerfile`;
- Run `docker build --tag kerblam_runtime .` to build the container;
- Run `docker run --rm -it -v ./data:/data kerblam_runtime`.

If you have your docker container `COPY . .` and have `ENTRYPOINT make`
(or `ENTRYPOINT bash`), you can then effectively have Kerblam! run your projects
in docker environments, so you can tweak your dependencies and tooling
(which might be different than your dev environment) and execute in a protected,
reproducible environment.

> :warning: When writing Dockerfiles, remember to `.dockerignore` the
> `./data/` folder, as it will be linked at runtime to `/data/`.

The same applies to `.sh` files in the `./src/pipes/` directory.

### Specifying data to run on
By default, Kerblam! will use your `./data/in/` folder as-is when executing pipes.
If you want the same pipes to run on different sets of input data, Kerblam! can
temporarily swap out your real data with this 'substitute' data during execution.

For example, your `process_csv.makefile` requires an input `./data/in/input.csv` file.
However, you might want to run the same pipe on another, `different_input.csv` file.
You could copy and paste the first pipe, modify it on every file you wish to
run differently. However, you then have to maintain two essentially identical
pipelines, and you are prone to adding errors while you do so.
You can use `kerblam` to do the same, but in a declarative, easy way.

Define in your `kerblam.toml` file a new section under `data.profiles`:
```toml
# You can use any ASCII name in place of 'alternate'.
[data.profiles.alternate]
# The quotes are important!
"input.csv" = "different_input.csv"
```
You can then run the same makefile with the new data with:
```
kerblam run process_csv --profile alternate
```
Under the hood, Kerblam! will:
- Rename `input.csv` to `input.csv.original`;
- Symlink `different_input.csv` to `input.csv`;
- Run the analysis as normal;
- When the run ends (or the analysis crashes), Kerblam! will remove the symlink
  and rename `<hex>_input.csv` back to `input.csv`.

This effectively causes the makefile run with different input data in this
alternate run.

> :warning: Careful that the *output* data will (most likely) be saved as the
> same file names as a "normal" run! Kerblam! does not look into where the
> output files are saved or what they are saved as.

> :warning: Careful! As of now, kerblam! has no problem overwriting existing
> files (e.g. `input.csv.original`) while running. See issue [#9](https://github.com/MrHedmad/kerblam/issues/9).

This is most commonly useful to run the pipelines on test data that is faster to
process or that produces pre-defined outputs. For example, you could define
something similar to:
```toml
[data.profiles.test]
"input.csv" = "test_input.csv"
"configs/config_file.yaml" = "configs/test_config_file.yaml"
```
And execute your test run with `kerblam run pipe --profile test`.

File paths specified under the `profiles` tab must be relative to the `./data/in/`
folder.

> :sparkles: Kerblam! tries its best to cleanup after itself (e.g. undo profiles,
> delete temporary files, etc...) when you use `kerblam run`, even if the pipe
> fails, and even if you kill your pipe with `CTRL-C`.

## Managing local and remote data
Kerblam! can help you retrieve remote data and manage your local data.

`kerblam data` will give you an overview of the status of local data:
```
> kerblam data
./data       500 KiB [2]
└── in       1.2 MiB [8]
└── out      823 KiB [2]
──────────────────────
Total        2.5 Mib [12]
└── cleanup  2.3 Mib [9] (92.0%)
└── remote   1.0 Mib [5]
! There are 3 undownloaded files.   
```
The first lines highlight the size (`500 KiB`) and amount (`2`) of files in the
`./data/in` (input), `./data/out` (output) and `./data` (intermediate) folders.

The total size of all the files in the `./data/` folder is then broken down
between categories: the `Total` data size, how much data can be removed with
`kerblam data clean` or `kerblam data pack`, and how many files are specified
to be downloaded but are not yet present locally.

You can manipulate your data with `kerblam data` in several ways.
In the following sections we explain every one of these ways.

### `kerblam data fetch` - Fetch remote data
If you define in `kerblam.toml` the section `data.remote` you can have
Kerblam! automatically fetch remote data for you:
```toml
[data.remote]
# This follows the form "url_to_download" = "save_as_file"
"https://raw.githubusercontent.com/MrHedmad/kerblam/main/README.md" = "some_readme.md"
```
When you run `kerblam data fetch`, Kerblam! will attempt to download `some_readme.md`
by following the URL you provided.
Most importantly, `some_readme.md` is treated as a file that is remotely available
and therefore locally expendable for the sake of saving disk size (see the
`data clean` and `data pack` commands).

You can specify any number of URLs and file names in `[data.remote]`, one for
each file that you wish to be downloaded.

### `kerblam data clean` - Free local disk space safely
If you want to cleanup your data (perhaps you have finished your work, and would
like to save some disk space), you can run `kerblam data clean`.
Kerblam! will remove:
- All temporary files in `./data/`;
- All output files in `./data/out`;
- All input files that can be downloaded remotely in `./data/in`.
This essentially only leaves input data that cannot be retrieved remotely on
disk.

Kerblam! will consider as "remotely available" files that are present in the
`data.remote` section of `kerblam.toml`.

### `kerblam data pack` - Package and export your local data
Say that you wish to send all your data folder to a colleague for inspection.
You can `tar -czvf exported_data.tar.gz ./data/` and send your whole data folder,
but you might want to only pick the output and non-remotely available inputs.

If you run `kerblam data pack` you can do just that.
Kerblam! will create a `exported_data.tar.gz` file and save it locally with the
non-remotely-available `.data/in` files and the files in `./data/out`.
You can also pass the `--cleanup` flag to also delete them after packing.

You can then share the data pack with others.

---

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)
