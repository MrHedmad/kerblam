![If you want it, Kerblam it!](docs/images/logo.png)

> :warning: **Most of this is not implemented yet.**
> Consider this README a roadmap of sort of what kerblam! wants to be.
>
> ```
>            new      run clone  data      ignore      link   tests
>              |        |     |     |           |         |       |
> [progress]>############------------------------------------------<
> ```

> :warning: `kerblam run` is complete but still untested. Please do use it,
> but be careful. Report any probles in the [issues](https://github.com/MrHedmad/kerblam).
> Thank you kindly!

Kerblam! is a tool that can help you manage data analysis projects.

A Kerblam! project has a `kerblam.toml` file in its root.
If you use `kerblam`, you then get some nice perks:
- Allows for easy remote data access, by just specifying URLs to fetch from;
- Can package and export data quickly to share the project with colleagues;
- Allows to manage and run multiple makefiles for different tasks;
- Leverages git to isolate, rollback and run the project at a different point in time;
- Cleans up intermediate and output files quickly;
- Manages Docker environments and runs code in them for you.
- Manage the content of you `.gitignore` for you, allowing to add files, 
  directories and even whole languages in one command.
- Helps you use `pre-commit` by managing `.pre-commit-hooks` for you.
- Allow you to specify test data to run.

To transform a project to a Kerblam! project just make the kerblam.toml
file yourself. To learn how, look at the section below.

## Overview

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
- :construction: `kerblam clone` can be used to clone a `kerblam` project.
  Kerblam! will ask you to fetch input files, create virtual environments and
  more upon creation.
- :pushpin: `kerblam data` fetches remote data and saves it locally, manages
  local data and can clean it up.
- :pushpin: `kerblam package` packages your pipeline and exports a `docker`
  image for execution later.
  It's useful for reproducibility purposes as the docker image is primed
  for execution, bundling the kerblam! executable, makefiles and non-remote
  data in the blob itself.
  Can also be used to just export the output data that the pipeline produces
  for sharing with others or for usage in writing reports/papers.
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

Say that you have a script in `./src/calc_sum.py`. It takes as input a `.csv`
and outputs a new `.csv` file as output, using `stdin` and `stdout`.
You have an `input.csv` file that you'd like to process, to create an
`output.csv`.
You could write a shell script or a makefile with the command to run.
We'll refer to these scripts as "pipes".
Here's an example makefile:

```makefile
./data/out/output.csv: ./data/in/input.csv ./src/calc_sum.py
    cat $< | ./src/calc_sum.py > $@
```

This is great, until you have to run another "pipe", completely different from
the first one, with different steps, requirements, etc...
You might write new makefiles or scripts for all the runs, but you'll then
have to remember how to structure each one, what each does and write a 
complex "meta"-makefile that runs the appropriate one.

Kerblam! manages your makefiles for you.
You can write different makefiles for different types of runs of your project,
and save them in `./src/pipes/`.
When you `kerblam run`, Kerblam! looks into that folder, finds (by name) the
makefiles that you've written, and brings them to the top level of the project
(e.g. `./`) for execution.

For instance, you could have written a `./src/pipes/process_csv.makefile` for
the previous step, and you could invoke it with `kerblam run process_csv`.
You could then write more makefiles for other tasks and run them similarly.

If Kerblam! finds a Dockerfile of the same name as one of your pipes in the
`./src/dockerfiles/` folder (e.g. `./src/dockerfiles/process_csv.dockerfile`),
it will:
- Move the dockerfile to the top folder, next to the makefile;
- Run `docker build --tag <name_of_makefile> .` to build the container;
- Run `docker run --rm -it -v ./data:/data <name_of_makefile>`.

If you have your docker container `COPY . .` and have `ENTRYPOINT make`
(or `ENTRYPOINT bash`), you can then effectively have Kerblam! run your projects
in docker environments, so you can tweak your dependencies and tooling
(which might be different than your dev environment).

> :warning: When writing Dockerfiles, remember to `.dockerignore` the
> `./data/` folder, as it will be linked at runtime to `/data/`.

The same applies to `.sh` files in the `./src/pipes/` directory.

### Specifying data to run on
By default, Kerblam! will use your whole `./data` folder
If you want different makefiles to run on different data, Kerblam! can
temporarily swap out your real data with this 'substitute' data.

For example, your `process_csv.makefile` requires an input `./data/in/input.csv` file.
However, you might want to run the same makefile on another, `different_input.csv` file.
You could copy and paste the first makefile, or you can use `kerblam` to do the same.
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
- Rename `input.csv` to `<hex>_input.csv`, where `<hex>` is a string with 5
  random numbers and letters;
- Symlink `different_input.csv` to `input.csv`;
- Run the analysis as normal;
- When the run ends (or the analysis crashes), Kerblam! will remove the symlink
  and rename `<hex>_input.csv` back to `input.csv`.

This effectively causes the makefile run with different input data in this
alternate run.

> :warning: Careful that the *output* data will be saved as the same file names
> as a "normal" run!

This is most commonly useful to run the pipelines on test data that is faster to
process or that produces pre-defined outputs. For example, you could define
something similar to:
```toml
[data.profiles.test]
"input.csv" = "test_input.csv"
"configs/config_file.yaml" = "configs/test_config_file.yaml"
```
And execute your test run with `kerblam run pipe --profile test`.

---

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)
