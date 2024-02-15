![If you want it, Kerblam it!](https://raw.githubusercontent.com/MrHedmad/kerblam/main/docs/images/logo.png)
<div align="center">

![GitHub issues](https://img.shields.io/github/issues/MrHedmad/kerblam?style=flat-square&color=blue)
![GitHub License](https://img.shields.io/github/license/MrHedmad/kerblam?style=flat-square)
![GitHub Repo stars](https://img.shields.io/github/stars/MrHedmad/kerblam?style=flat-square&color=yellow)
[![All Contributors](https://img.shields.io/github/all-contributors/MrHedmad/kerblam?color=ee8449&style=flat-square)](docs/CONTRIBUTING.md)
[![DOI](https://zenodo.org/badge/720446939.svg?style=flat-square)](https://zenodo.org/doi/10.5281/zenodo.10664806)

</div>

> [!WARNING]
>
> Kerblam! is still unreleased. Expect some bugs, glitches or breaking changes.
> Always have a backup of your data and code!
> Report any problems in the [issues](https://github.com/MrHedmad/kerblam).
> Thank you for using Kerblam!

Kerblam! is a tool that can help you manage data analysis projects.

A Kerblam! project has a `kerblam.toml` file in its root.
Kerblam! then allows you to:
- Access remote data quickly, by just specifying URLs to fetch from;
- Package and export data in order to share the project with colleagues;
- Manage and run multiple makefiles or shellfiles for different tasks;
- Clean up intermediate and output files quickly;
- Manage containers and run code in them for you.
- Manage the content of your `.gitignore` for you, allowing to add files, 
  directories and even whole languages in one command.
- Specify test or alternative data and quickly use it instead of real data.

To transform a project to a Kerblam! project just make the kerblam.toml
file yourself. To learn how, look at the section below.

# Overview
Kerblam! has the following commands:
- :magic_wand: `kerblam new` can be used to create a new kerblam!
  project. Kerblam! asks you if you want to use some common programming
  languages and sets up a proper `.gitignore` for you.
- :package: `kerblam data` fetches remote data and saves it locally, manages
  local data and can clean it up, preserving only files that must be preserved.
  It also shows you how much local data is on the disk, how much data is remote and
  how much disk space you can free without losing anything important.
  It can also export the important data to share it with colleagues quickly.
- :gift: `kerblam package` packages your pipelines and exports a container
  image for execution later.
  It's useful for reproducibility purposes as the docker image is primed
  for execution, bundling the kerblam! executable, execution files and non-remote
  data in the blob itself.
- :rocket: `kerblam run` executes the analysis for you,
  by choosing your `makefile`s and containers appropriately and 
  building container images as needed.
  Optionally, allows test data or alternative data to be used instead of
  real data, in order to test your pipelines.
- :scissors: `kerblam ignore` can edit your `.gitignore` file by adding files,
  folders and GitHub's recommended ignores for specific languages in just one command.

Kerblam! is *not* and does not want to be:
- A pipeline manager like `snakemake` and `nextflow`: It supports and helps
  you execute `make`, but it does not interfere from then on;
  - You can however use `snakemake`, `nextflow` or any other program in conjunction
    with Kerblam!
- A replacement for any of the tools it leverages (e.g. `git`, `docker` or `podman`,
  `pre-commit`, etc...);
- Something that insulates you from the nuances of writing good, correct
  pipelines and container recipies.\
  Specifically, Kerblam! will never:
  - Parse your `.gitignore`, `.dockerignore`, pipes or container recipies to check
    for errors or potential issues;
  - Edit code for you (with the exception of a tiny bit of wrapping to allow
    `kerblam package` to work);
  - Handle any errors produced by the pipelines or containers.
- A tool that covers every edge case. Implementing more features for popular
  and widespread tasks is perfectly fine, but Kerblam! will never have a wall
  of options for you to choose from.
  If you need more advanced control on what is done, you should directly
  use the tools that Kerblam! leverages.

## Opinions
Kerblam! projects are opinionated:
- The folder structure of your project adheres to the Kerblam! standard,
  although you may configure it in `kerblam.toml`.
  Read about it below.
- You use `make` or bash scripts as your pipeline manager.
  - Kerblam! natively uses `make`, but nothing stops you writing
    shell files that execute other tools, like `snakemake`.
- You use `docker` or `podman` as your virtualisation service.
- You use `git` as your version control system.
- You generally execute your pipelines in a container, and not in your development
  environment.
- Most of your input data is remotely downloadable, especially for large and
  bulky files.

If you don't like this setup, Kerblam! is probably not for you.

> [!TIP]
> If you wish to learn more on why these design choices were made, please
> take a look at [the kerblam! philosophy](docs/philosophy.md).

### Folder structure
Kerblam!, by default, requires the following folder structure (relative to the
location of the `kerblam.toml` file):
- `./kerblam.toml`: This file contains the options for Kerblam!. It is often empty.
- `./data/`: This is a directory for the data. Intermediate data files are saved here.
- `./data/in/`: Input data files are saved and should be looked for here.
- `./data/out/`: Output data files are saved and should be looked for here.
- `./src/`: Code you want to be executed should be saved here.
- `./src/pipes/`: Makefiles and bash build scripts should be saved here.
  They have to be written as if they were saved in `./`.
- `./src/dockerfiles/`: Container build scripts should be saved here. 

You can configure almost all of these paths in `kerblam.toml`, if you so desire.
This is mostly done for compatibility reasons with non-kerblam! projects.
New projects that wish to use Kerblam! are strongly encouraged to follow the
standard folder structure, however.

## Documentation
The full Kerblam! documentation is in the [`/docs` folder](docs/README.md).
Please take a look there for more information on what Kerblam! can do.
For example, you might find [the tutorial](docs/tutorial.md) interesting.

## Installation
Currently, Kerblam! only supports mac OS (both intel and apple chips) and GNU linux.
Other unix/linux versions may work. Install them from source with the commands below.

### Requirements
Kerblam! requires a Linux (or generally unix-like) OS.
It also uses binaries that it assumes are already installed:
- GNU `make`: https://www.gnu.org/software/make/
- `git`: https://git-scm.com/
- Docker (as `docker`) and/or Podman (as `podman`): https://www.docker.com/ and https://podman.io/
- `tar`.

If you can use `git`, `make`, `tar` and `docker` or `podman` from your CLI, you should be good.

### Pre-compiled binary (recommended)
You can find and download a Kerblam! binary for your operating system in
[the releases tab](https://github.com/mrhedmad/kerblam/releases).

There are also helpful scripts that automatically download the correct version
for your specific operating system thanks to [`cargo-dist`](https://github.com/axodotdev/cargo-dist).
You can always install or update to the latest version with:
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MrHedmad/kerblam/releases/latest/download/kerblam-installer.sh | sh
```
You can [click here](https://github.com/MrHedmad/kerblam/releases/latest/download/kerblam-installer.sh)
to download the same installer script and inspect it before you run it, if you'd like.

### Install from source
If you want to install the latest version from source, install Rust and `cargo`, then run:
```bash
cargo install kerblam
```
If you wish to instead use the latest development version, run:
```bash
cargo install --git https://github.com/MrHedmad/kerblam.git
```
The `main` branch should always compile on supported platforms with the above command.
If it does not, please [open an issue](https://github.com/mrhedmad/kerblam/issues/new).

## Adding the Kerblam! badge
You can add a Kerblam! badge in the README of your project to show that you use Kerblam!
Just copy the following code and add it to the README:
```markdown
![Kerblam!](https://img.shields.io/badge/Kerblam!-v0.4.0-blue?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC0AAAAtCAMAAAANxBKoAAABlVBMVEUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADW1tYNDHwcnNLKFBQgIB/ExMS1tbWMjIufDQ3S0tLOzs6srKyioqJRUVFSS0o0MjIBARqPj48MC3pqaWkIB2MtLS1ybm3U1NS6uroXirqpqamYmJiSkpIPZ4yHh4eFhIV8fHwLWnuBe3kMC3cLCnIHBlwGBlgFBU8EBEVPRkICAi4ADRa+EhIAAAwJCQmJiYnQ0NDKysoZkMK2trYWhLOjo6MTeKMTd6KgoKCbm5uKiIaAgIAPDHhubm4JT20KCW0KCWoIS2cHBUxBQUEEAz9IQT4DAz0DKTpFPTgCAjcCASoBASAXFxcgGRa5ERG1ERGzEBCpDw+hDg4fFA2WDAyLCgouAQFaWloFO1MBHStWBATnwMkoAAAAK3RSTlMA7zRmHcOuDQYK52IwJtWZiXJWQgXw39q2jYBgE/j2187JubKjoJNLSvmSt94WZwAAAvlJREFUSMeF1GdXGkEUgOGliIgIorFH0+u7JBIChEgJamyJvWt6783eS8rvzszAusACvp88x4d7hsvsaqdU57h8oQnobGmtb6xMzwbOkV9jJdvWBRwf7e9uLyzs7B3+o7487miC+AjcvZ3rkNZyttolbKxPv2fyPVrKYKcPhp7oIpPv0FkGN5N5rmd7afAFKH0MH99DihrTK2j3RTICF/Pt0trPUr9AxXyXpkJ3xu6o97tgQJDQm+Xlt6E8vs+FfNrg6kQ1pOuREVSPoydf9YjLpg14gMW1X0IInGZ+9PWr0Xl+R43pxzgM3NgCiekvqfE50hFdT7Ly8Jbo2R/xWYNTl8Ptwk6lgsHUD+Ji2NMlBFZ8ntzZRziXW5kLZsaDom/0yH/G+CSkapS3CvfFCWTxJZgMyqbYVLtLMmzoVywrHaPrrNJX4IHCDyCmF+nXhHXRkzhtCncY+PMig3pu0FfzJG900RBNarTTxrTCEwne69miGV5k8cPst3wOHSfrmJmcCH6Y42NEzzXIX8EFXmFE/q4ZXJrKW4VsY13uzqivF74OD39CbT/0HV/1yQW9Xn8e1O0w+WAG0VJS4P4Mzc7CK+2B7jt6XtFYMhl7Kv4YWMKnsJkXZiW3NgQXxTEKamM2fL8EjzwGv1srykZveBULj6bBZX2Bwbs03cXTQ3HAb9FOGNsS4wt5fw9zv0q9oZo54Gf4UQ95PLbJj/E1HFZ9DRgTuMecPgjfUqlF7Jo1B9wX+JFxmMh7mAoGv9B1pkg2tDoVl7i3G8mjH1mUN3PaspJaqM1NH/sJq2L6QJzEZ4FTCRosuKomdxjYSofDs8DcRPZh8hQd5IbE3qt1ih+MveuVeP2DxOMJAlphgSs1mt3GVWO6yMNGUDZDi1uzJLDNqxbZDLab3mqQB5mExtLYrtU45L10qlfMeSbVQ91eFlfRmnclZyR2VcB5y7pOYhouuSvg2rxHCZG/HHZnsVkVtg7NmkdirS6LzbztTq1EPo9dXRWxqtP7D+wL5neoEOq/AAAAAElFTkSuQmCC&link=https%3A%2F%2Fgithub.com%2FMrHedmad%2Fkerblam)
```
> [!WARNING]
> The code is very long - this is because the Kerblam! logo is baked in as a `base64` image.

## Contributing
To contribute, please take a look at [the contributing guide](docs/CONTRIBUTING.md).

Code is not the only thing that you can contribute.
Written a guide? Considered a new feature? Wrote some docstrings? Found a bug?
All of these are meaningful and important contributions.
For this reason, **all** contributors are listed in
[the contributing guide](docs/CONTRIBUTING.md).

If you use Kerblam! or want to add your opinion to the direction it is taking,
take a look at [the issues labelled with RFC](https://github.com/MrHedmad/kerblam/issues?q=is%3Aissue+is%3Aopen+label%3ARFC).
They are *requests for comments* where you can say your opinion on new features.

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

---

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)
