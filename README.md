![If you want it, Kerblam it!](docs/images/logo.png)
<div align="center">

![GitHub issues](https://img.shields.io/github/issues/MrHedmad/kerblam?style=flat-square&color=blue)
![GitHub License](https://img.shields.io/github/license/MrHedmad/kerblam?style=flat-square)
![GitHub Repo stars](https://img.shields.io/github/stars/MrHedmad/kerblam?style=flat-square&color=yellow)
[![All Contributors](https://img.shields.io/github/all-contributors/MrHedmad/kerblam?color=ee8449&style=flat-square)](docs/CONTRIBUTING.md)

</div>

> [!WARNING]
>
> `kerblam run` and `kerblam package` are complete but still untested.
> Please do use them, but be careful.
> Always have a backup of your data and code!
> Report any problems in the [issues](https://github.com/MrHedmad/kerblam).
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
- Manage the content of your `.gitignore` for you, allowing to add files, 
  directories and even whole languages in one command.
- Make it easy to use `pre-commit` by managing `.pre-commit-hooks`.
- Specify test data to run and quickly use it instead of real data.

To transform a project to a Kerblam! project just make the kerblam.toml
file yourself. To learn how, look at the section below.

# Overview

> [!WARNING]
>
> Some commands are missing some features that would be nice to have.
> Please take a look at the [issues](https://github.com/MrHedmad/kerblam/issues)
> and see if what you'd like to do is already proposed and/or being worked on.
> If you don't find it, open an issue yourself detailing what you think would
> be a good addition!

- :white_check_mark: `kerblam new` can be used to create a new kerblam!
  project. Kerblam! asks you if you want to use some common programming
  languages and sets up a proper `.gitignore` and pre-commit hooks for you.
- :white_check_mark: `kerblam data` fetches remote data and saves it locally, manages
  local data and can clean it up, preserving only files that must be preserved.
  It also shows you how much local data is on the disk, how much data is remote and
  how much disk space you can free without losing anything important.
- :white_check_mark: `kerblam package` packages your pipeline and exports a `docker`
  image for execution later.
  It's useful for reproducibility purposes as the docker image is primed
  for execution, bundling the kerblam! executable, execution files and non-remote
  data in the blob itself.
- :white_check_mark: `kerblam run` executes the analysis for you,
  by choosing your `makefile`s and `dockerfiles` appropriately and 
  building docker containers as needed.
  Optionally, allows test data or alternative data to be used instead of
  real data, in order to test your pipelines.
- :white_check_mark: `kerblam ignore` can edit your `.gitignore` file by adding files,
  folders and GitHub's recommended ignores for specific languages in just one command.

Kerblam! is *not* and does not want to be:
- A pipeline manager like `snakemake` and `nextflow`: It supports and helps
  you execute `make`, but it does not interfere from then on;
- A replacement for any of the tools it leverages (e.g. `git`, `docker`,
  `pre-commit`);
- Something that insulates you from the nuances of writing good, correct
  pipelines and Dockerfiles.\
  Specifically, Kerblam! will never:
  - Parse your `.gitignore`, `.dockerignore`, pipes or `Dockerfile`s to check
    for errors or potential issues;
  - Edit code for you (with the exception of a tiny bit of wrapping to allow
    `kerblam package` to work);
  - Handle any errors produced by the pipelines or containers.
- A tool that covers every edge case. Implementing more features for popular
  and widespread tasks is perfectly fine, but Kerblam! will never have a wall
  of options for you to choose from.
  If you need more advanced control on what is done, you should directly
  use the tools that Kerblam! leverages.

> [!TIP]
>
> Kerblam! works *with* you, not *for* you!

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
- Most of your input data is remotely downloadable, especially for large and
  bulky files.

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

You can configure all of these paths in `kerblam.toml`, if you so desire.
This is mostly done for compatibility reasons with non-kerblam! projects.

> [!WARNING]
> Please take a look at issue #11 before editing your paths.

## Contributing
To contribute, please take a look at [the contributing guide](docs/CONTRIBUTING.md).

Code is not the only thing that you can contribute.
Written a guide? Considered a new feature? Wrote some docstrings? Found a bug?
All of these are meaningful and important contributions.
For this reason, **all** contributors are listed in
[the contributing guide](docs/CONTRIBUTING.md).

Thank you for taking an interest in Kerblam! Any help is really appreciated.

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
- `tar`.

If you can use `git`, `make`, `tar` and `docker` from your CLI, you should be good.

## Documentation
The Kerblam! documentation is in the [`/docs` folder](docs/README.md).
Please take a look there for more information on what Kerblam! can do.
For example, you might find [the tutorial](docs/tutorial.md) interesting.

---

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)
