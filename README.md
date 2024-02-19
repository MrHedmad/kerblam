[![If you want it, Kerblam it!](https://raw.githubusercontent.com/MrHedmad/kerblam/main/docs/images/logo.png)](https://kerblam.dev/)
<div align="center">

![GitHub issues](https://img.shields.io/github/issues/MrHedmad/kerblam?style=flat-square&color=blue)
![GitHub License](https://img.shields.io/github/license/MrHedmad/kerblam?style=flat-square)
![GitHub Repo stars](https://img.shields.io/github/stars/MrHedmad/kerblam?style=flat-square&color=yellow)
[![All Contributors](https://img.shields.io/github/all-contributors/MrHedmad/kerblam?color=ee8449&style=flat-square)](CONTRIBUTING.md)
[![DOI](https://zenodo.org/badge/720446939.svg?style=flat-square)](https://zenodo.org/doi/10.5281/zenodo.10664806)

</div>

<div align="center">

[🚀 Read the full Kerblam Documentation 🚀](https://kerblam.dev)

</div>

Kerblam! is a tool that can help you manage
[data analysis projects](https://en.wikipedia.org/wiki/Data_analysis).
A Kerblam! project has a `kerblam.toml` file in its root.
Kerblam! then allows you to:
- Manage and run multiple makefiles or shellfiles for different tasks;
- Manage containers and run code in them for you;
- Create reproducible containers to replay your work or share it with others;
- Access remote data quickly, by just specifying URLs to fetch from;
- Specify test or alternative data and quickly use it instead of real data.
- Package and export data in order to share the project with colleagues;
- Clean up intermediate and output files quickly;
- Manage the content of your `.gitignore` for you, allowing to add files, 
  directories and even whole languages in one command.

To transform a project to a Kerblam! project you can just make the `kerblam.toml`
file yourself. [Read the documentation to learn how](https://kerblam.dev/).
If you want to see how Kerblam! works in action, you can
[see usage examples of Kerblam! ✨](https://github.com/MrHedmad/kerblam-examples)

# Overview
Kerblam! has the following commands:
- :magic_wand: `kerblam new` can be used to create a new kerblam!
  project. Kerblam! asks you if you want to use some common programming
  languages and sets up a proper `.gitignore` for you.
- :rocket: `kerblam run` executes the analysis for you,
  by choosing your `makefile`s and containers appropriately and 
  building container images as needed.
  Optionally, allows test data or alternative data to be used instead of
  real data, in order to test your pipelines.
- :gift: `kerblam package` packages your pipelines and exports a container
  image for execution later.
  It's useful for reproducibility purposes as the docker image is primed
  for execution, bundling the kerblam! executable, execution files and non-remote
  data in the blob itself.
- :package: `kerblam data` fetches remote data and saves it locally, manages
  local data and can clean it up, preserving only files that must be preserved.
  It also shows you how much local data is on the disk, how much data is remote and
  how much disk space you can free without losing anything important.
  It can also export the important data to share it with colleagues quickly.
- :scissors: `kerblam ignore` can edit your `.gitignore` file by adding files,
  folders and GitHub's recommended ignores for specific languages in just one command.

Kerblam! is *not* and does not want to be:
- :non-potable_water: A pipeline manager like `snakemake` and `nextflow`;
  - It supports and helps you execute pipelines written in other formats, but
    it does not interfere from then on;
  - You can however use `snakemake`, `nextflow` or any other program in conjunction
    with Kerblam! by writing shell pipelines.
- :recycle: A replacement for any of the tools it leverages (e.g. `git`, `docker` or `podman`,
  `pre-commit`, etc...);
- :mag_right: Something that insulates you from the nuances of writing good, correct
  pipelines and container recipies.
  Specifically, Kerblam! will never:
  - Parse your `.gitignore`, `.dockerignore`, pipes or container recipies to check
    for errors or potential issues;
  - Edit code for you (with the exception of a tiny bit of wrapping to allow
    `kerblam package` to work);
  - Handle any errors produced by the pipelines or containers.
- :earth_africa: A tool that covers every edge case.
  Kerblam! will never have a wall of options for you to choose from.
  If you need more advanced control on what is done, you should directly
  use the tools that Kerblam! leverages.

## Opinions
Kerblam! wants to streamline and standardize data analysis project as much as
possible. For this reason, projects are opinionated:
- The folder structure of your project adheres to the Kerblam! standard,
  although you may configure it in `kerblam.toml`.
  Read more about it [here](https://kerblam.dev/quickstart.md).
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
> take a look at [the kerblam! philosophy](html://kerblam.dev/philosophy.html).

## Documentation
The full Kerblam! documentation is online at [kerblam.dev](https://kerblam.dev).
Please take a look there for more information on what Kerblam! can do.
For example, you might find [the tutorial](https://kerblam.dev/quickstart.html) interesting.

## Installation

<div align="center">

[✨ Read the full installation guide in the docs ✨](https://kerblam.dev/install.html)

</div>

In short, use a unix-compatible OS and either: 
```bash
# Install a prebuild binary
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MrHedmad/kerblam/releases/latest/download/kerblam-installer.sh | sh
```
or, alternatively
```bash
# Install from source with rustc and cargo
cargo install kerblam
```
You will need `git`, `make`, `tar` and `docker` or `podman` installed for
Kerblam! to work.

## Contributing
To contribute, please take a look at [the contributing guide](CONTRIBUTING.md).

Code is not the only thing that you can contribute.
Written a guide? Considered a new feature? Wrote some docstrings? Found a bug?
All of these are meaningful and important contributions.
For this reason, **all** contributors are listed in [the contributing guide](CONTRIBUTING.md).

If you use Kerblam! or want to add your opinion to the direction it is taking,
take a look at [the issues labelled with RFC](https://github.com/MrHedmad/kerblam/issues?q=is%3Aissue+is%3Aopen+label%3ARFC).
They are *requests for comments* where you can say your opinion on new features.

Thank you for taking an interest in Kerblam! Any help is greatly appreciated.

## Licensing and citation
Kerblam! is licensed under the [MIT License](https://github.com/MrHedmad/kerblam/blob/main/LICENSE).
If you wish to cite Kerblam!, please provide a link to this repository or use
the Zenodo DOI [10.5281/zenodo.10664806](https://zenodo.org/doi/10.5281/zenodo.10664806)

## Naming
This project is named after the fictitious online shop/delivery company in
[S11E07](https://en.wikipedia.org/wiki/Kerblam!) of Doctor Who.
Kerblam! might be referred to as Kerblam!, Kerblam or Kerb!am, interchangeably,
although Kerblam! is preferred.
The Kerblam! logo is written in the [Kwark Font](https://www.1001fonts.com/kwark-font.html)
by [tup wanders](https://www.1001fonts.com/users/tup/).

---

<div align="center">

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)

</div>
