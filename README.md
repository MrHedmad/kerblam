[![If you want it, Kerblam it!](https://raw.githubusercontent.com/MrHedmad/kerblam/main/docs/images/logo.png)](https://kerblam.dev/)
<div align="center">

![GitHub issues](https://img.shields.io/github/issues/MrHedmad/kerblam?style=flat-square&color=blue)
![GitHub License](https://img.shields.io/github/license/MrHedmad/kerblam?style=flat-square)
![GitHub Repo stars](https://img.shields.io/github/stars/MrHedmad/kerblam?style=flat-square&color=yellow)
[![All Contributors](https://img.shields.io/github/all-contributors/MrHedmad/kerblam?color=ee8449&style=flat-square)](CONTRIBUTING.md)\
[![DOI](https://zenodo.org/badge/720446939.svg?style=flat-square)](https://zenodo.org/doi/10.5281/zenodo.10664806)

</div>

<div align="center">

[🚀 Read the full Documentation 🚀](https://kerblam.dev)
|
[✨ See usage Examples ✨](https://github.com/MrHedmad/kerblam-examples)

</div>

## Kerblam! is a project management system.

Wherever you have input data that needs to be processed to obtain some output,
Kerblam! can help you out by dealing with the more tedious and repetitive parts
of working with data for you, letting you concentrate on getting things done.

You can [watch a demo of what Kerblam! can do for you](https://asciinema.org/a/641448),
or you can jump right in by [reading the quickstart guide](https://kerblam.dev/quickstart.html).

If you are here often, you might need [the installation command](https://kerblam.dev/install.html).

# Overview
These are some of the features of Kerblam!:
- :magic_wand: You can quickly start a new project with `kerblam new`.
  Kerblam generates the folder structure of your project using sensible defaults,
  similar to what [`cookiecutter`](https://github.com/cookiecutter/cookiecutter) does.
- :rocket: Kerblam! can manage your workflows, written for different
  workflow management systems. It chiefly prefers GNU `make`, but can manage
  anything that can be executed via the command line.
- :recycle: Kerblam! can hotswap input data just before your workflows start,
  letting you work with different sets of input data without having to touch
  the configuration of your workflows.
  For example, this is very useful to temporarily work with test data.
- :gift: Kerblam! lets you run arbitrary workflows into containers, dealing
  with volume mounting and other details for you.
  This lets you package workflows that are not natively containerized
  into Docker for greater reproducibility.
  For instance, you might just need to run a very tiny data processing pipeline,
  and don't want to deal with the verbosity of robust workflow management
  systems like Snakemake, CWL or Nextflow.
- :package: `kerblam package` packages your pipelines and exports a container
  image for execution later. This leaves a tarball with all the data a
  reproducer needs to run the analysis again, so they may do so quickly and
  easily. The reproducer may do so manually, or use Kerblam! again to
  un-package the project.
- :package: Kerblam! can fetch remote data and save it locally, manage
  local data and clean it up, preserving only files that must be preserved.
  It also shows you how much local data is on the disk, how much data is remote and
  how much disk space you can free without losing anything important.
  It can also export the important data to share it with colleagues quickly.

Kerblam! is *not* and *does not want to be*:
- :non-potable_water: A workflow runner like `snakemake`, `nextflow` or `cwltool`;
  - It supports and helps you execute pipelines written *for* other WMS, but
    it does not interfere from then on;
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

## Opinions
Kerblam! wants to streamline and standardize data analysis projects as much as
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
The full Kerblam! documentation is online at [kerblam.dev 🚀](https://kerblam.dev).
Please take a look there for more information on what Kerblam! can do.

You might find [the tutorial](https://kerblam.dev/quickstart.html) interesting.

## Installation

<div align="center">

[✨ Read the full installation guide in the docs ✨](https://kerblam.dev/install.html)

</div>

> [!WARNING]
> Release candidates (e.g. `1.0.0-rc.x`) are not available on crates.io! Use the pre-compiled
> binaries in the releases tab, the command below or compile directly from git with
> `cargo install --git https://github.com/MrHedmad/kerblam`.

In short, use a unix-compatible OS and either: 
```bash
# Install a prebuilt binary
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MrHedmad/kerblam/releases/latest/download/kerblam-installer.sh | sh
```
or, alternatively
```bash
# Install from source with rustc and cargo
cargo install kerblam
```
You will need `git`, `make` and `docker` or `podman` installed for Kerblam! to work.

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
Kerblam! might be referred to as Kerblam!, Kerblam or Kerb!am, interchangeably, although Kerblam! is preferred.
The Kerblam! logo is written in the [Kwark Font](https://www.1001fonts.com/kwark-font.html)
by [tup wanders](https://www.1001fonts.com/users/tup/).

---

<div align="center">

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)

</div>
