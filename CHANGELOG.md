# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Possible sections are "Added", "Changed", "Deprecated", "Removed", "Fixed" and
"Security".

Versions are listed in reverse chronological order, with the most recent at
the top. Non pre-release versions sometimes have an associated name.

## [Unreleased]
### New
- The error message you get when running `kerblam run` with no parameters now
  includes a list of available profiles, or tells you that you have specified
  no profiles.

## [v1.0.0-rc.3] - 2024-06-24

This release adds a few tweaks that should be made now before 1.0.0 and we
stop being retrocompatible

### Changed
- **BREAKING CHANGE**: The default path for workflows was changed from 
  `src/pipes` to `src/workflows`.
  - The "pipeline" terminology is apparently quite old, and therefore
    "workflow" sounds more natural to the modern ear.
- The book and README were quite heavily updated to reflect the nature of
  Kerblam!. A quickstart project was added to get people started with an
  hand-on approach.

### Fixed
- The `test` profile could not be used if the `data.profiles` section of the
  TOML was not there (even if empty).
  This was fixed by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/104
- If you deleted some profile files between profiled runs, the cache would
  tell Kerblam! to touch the files, but it would fail, since they are not
  found.
  Now, Kerblam! just silently ignores these files - if they are no there,
  there is no need to update their timestamps, is there?
  Fixed by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/102

## [v1.0.0-rc.2] - 2024-05-28
This release candidate adds a few fixes and features, and it is probably the
closest to a real release of Kerblam! that we will get.

## What's Changed
### Added
- `kerblam run` now accepts command-line arguments to be passed directly to the
  workers that it spawns (i.e. `make` or `bash`) by using two dash separators.
  - e.g. `kerblam run ... -- arg1 arg2`
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/88
- Kerblam! now caches the name of the last profile used, so that it can skip
  updating timestamps if the same profile is used back-to-back.
  This allows `make` to properly skip rebuilding files that should not be
  rebuilt when using profiles 
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/89
- The book on [kerblam.dev](https://kerblam.dev) now uses callouts thanks to
  [`mdbook-callouts`](https://crates.io/crates/mdbook-callouts)
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/95
- Added workflows community metadata and icons.
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/98

### Fixed
- `make` properly changes working directory following kerblam.toml in docker
  by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/92
- Perfect some aspects of directory-based profiles
  by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/94
  - Profiles that act on directories were already available, but some issues
    were linked with using them, like timestamp updating.
    Now they have been fixed, and documentation was added on how to use them.
- The documentation was updated in many aspects.
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/85
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/86
  - @MrHedmad in https://github.com/MrHedmad/kerblam/pull/93

## [v1.0.0-rc.1] - 2024-02-26
This release adds the `replay` command and changes quite dramatically how the
containers are packaged.

The documentation was also updated quite thoroughly.
The PR for this release was #69 by @MrHedmad.

### Added
- The `kerblam replay` command was added. It takes a package tarball,
  reads the `kerblam.toml` file to find out where the input files were,
  and unpacks them. It then takes the name of the docker container and runs
  it with the proper bindings.
  - You can do all of this manually, but it's much more convenient to
    have Kerblam! do it for you.

### Changed
- ! BREAKING - The `kerblam package` command works differently:
  - The data is no longer included in the package;
  - A new tarball is created with the (precious-only) input data, the name
    of the container (in a `name` file) and the `kerblam.toml`.
  - The entrypoint was changed from `kerblam data fetch && make ...` or
    `kerblam data fetch && bash`... to simply
    `kerblam data fetch && kerblam run <packaged pipeline>`.

The documentation was also updated quite thoroughly. The PR for this release was #69 by @MrHedmad.


## [v1.0.0-rc.0] - 2024-02-18
- Nothing.

## [v0.5.1] - 2024-02-16

### Fixed 
- Fix bad parsing of root dir path by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/63
  - The default path ("/") was not applied correctly to internal (host) paths
    since Rust's PathBuf treats it as an empty path. This was fixed.
  - Reverted a breaking change whereas the `src/dockerfiles` default path was
    changed to `src/containers`, but this was not supposed to be released yet.

## [v0.5.0] - 2024-02-15 - Gears an Wrenches
This release mainly changes stuff at the backend of Kerblam! making it more
streamlined for future development.
However, there are some small features that slipped through the cracks in the other versions.

### Added
- Add tiny features by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/60
  - Markdown (esque) rendering of pipe descriptions with `kerblam run pipe --desc`
  - Allow Kerblam! to use a default dockerfile (aptly named `default.dockerfile`) for
    all pipes if no specific dockerfile is found.
    The container icon (a :whale:) is swapped out for a more modest :fish: if
    a pipeline uses the default dockerfile.
- Do not overwrite any files in profiles (#9) by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/61
  - Potentially useless, but it's there now.
    If you have a `file.txt.original` (for some reason) and kerblam!
    would overwrite it, it now stops before doing something potentially destructive. 
- Add Tests to Kerblam! by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/62
  - Kerblam! has shiny new tests! They don't cover much for now, but they provide the grounds to write more.
  - Taken the time to refactor functions to be more testable.

### Changed
- Code has been refactored quite heavily by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/60

## [v0.4.0] - 2024-02-01 - Much too convenient
In this *much too convenient* release, I've added a few QOL changes that were
in the issues for a long time.
You can stop writing your own entrypoints in dockerfiles (convenient),
you can change the dockerized working directory in the `kerblam.toml` file
(super convenient!), tell your pipelines that they are being profiled
(very... convenient for them), cleanup your empty directories
(suspiciously convenient) and use Podman instead of Docker
(for a more convenient local execution).

### Added
- Add a way to show long descriptions of pipes by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/50
  - `kerblam run my_pipe --desc` will print out the full description of a pipe (if there is one). 
- Infer test profile automatically given file names by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/52
  - If `[profiles.test]` is not defined, Kerblam! will make one up by swapping all `test_xxx` files with `xxx` files. Convenient!
- Set the `KERBLAM_PROFILE` env variable when in a profile by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/53
  - Pipes can now be aware they are in a profile, an act accordingly.
- Warn the user if they fetch to a file not in the input data dir by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/54
  - If you mistakenly add a `/` to the start of your retrieved file, kerblam! will warn you before doing something you might regret.
- Automatically find kerblam.toml in parent dirs by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/56
  - You can Kerblam! almost anywhere now!
- Add podman support by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/57
  - If you like FOSS options now you can use [Podman](https://podman.io) instead of Docker as your container runner of choice.

### Changed
- Overhaul containerized execution by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/58
  - Kerblam! will set the correct `ENTRYPOINT` so you don't have to set it yourself anymore. Super convenient!
  - You can now tell kerblam! if you are packaging your pipeline in anywhere other than the root of the container, so you can keep everything separated nicely.
- Update documentation with new features by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/59
  - There are several new features, and the docs are updated to reflect this. Read them again if you want the full picture!
- Cleaning data files now removes empty directories left behind by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/55
  - Your `/data` folder will be squeaky clean. Suppress this with the `--keep-dirs` flag.

### Fixes
- Ask again if user types nothing as an answer (#49) by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/51
  - Kerblam! used to crash if you just pressed enter at one of its prompts. No longer! It will **demand** an answer from you - forever!
- Drop openssl requirement by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/43
  - Sorry if you had trouble installing kerblam! due to this. Gets rid of a series of `missing libssl.1.1` errors on various OSes.


## [v0.3.0] - 2024-01-24 - A light in the dark

### Added
- Add kerblam! Shields badge by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/34
  - You can now add a snazzy badge with the kerblam!
    rocket and the version of Kerblam! of your project (all manually updated for now).
- Show the list of available pipes even when none are typed by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/35
  - This means that `kerblam run` with no specified pipeline shows you
    the list of available pipes, just like you get when you misspell a pipeline.
    No more `kerblam run asd` to see the list! 
- Available pipes message includes description by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/41
  - You can now include descriptions to your pipes.

### Fixed
- Switching profiles correctly updates the file access metadata by @MrHedmad
  in https://github.com/MrHedmad/kerblam/pull/36
  - When you used a `--profile`, `make` did not realize that anything had
    changed. Now it does, as you'd expect it to.

## [v0.2.1] - 2024-01-08 - The --version version

### Added
- Add a `--version` flag to print version string by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/30

### Fixed
- Fix wrong remote data files path by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/32

## [v0.2.0] - 2024-01-03 - The nice things update
There are a lot of features that I noticed are immediately nice to have.
This update brings many of them to Kerblam! to make it much more ergonomic to use.
It also includes quite a bit of fixes and some important under-the-hood changes.

### Added
- Add version compatibility check by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/23
  - Kerblam! now complains if the version under `meta > version` is not the
    same as the current Kerblam! version, to save you some headaches due
    to incompatibility.
- Show available pipes on failed run/package command by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/24
  - Kerblam! now shows you all the pipes it can find if you type a pipe that
    does not exist when you `kerblam run` or `kerblam package`.
    This should save a bunch of time if you cannot remember if the pipe was
    named `execute`, `compute` or `calculate`.
- Add `--keep-remote; option to 'data clean' by @MrHedmad in https://github.com/MrHedmad/kerblam/pull/29
  - Sometimes it's good to quickly start over.
    With `kerblam data clean --keep-remote` you can cleanup all generated data
    but keep the remote files so you don't have to re-fetch them before
    running again.
- Add a way to force running locally (@MrHedmad  in 87c39f040206a8ae3a5ce9b13f904c1f125c2aa4)
  - If you have a dockerfile does not mean you want to use it **all** the time.
    `kerblam run --local` skips using the container and runs the pipeline
    locally, even if a corresponding dockerfile is found.
    This should be useful during development.

### Fixed
- Issue #11 should be fixed now, but testing is still required (@MrHedmad in c8ea7051a1538fdf6ed1d9ecad011e1ea0a5347e).
  - This means that setting paths in the `kerblam.toml` file should be working as intended.


## [v0.1.0] - 2023-12-07 - The beginning

### Added
- `kerblam new` can be used to create a new kerblam! project.
  Kerblam! asks you if you want to use some common programming languages and
  sets up a proper `.gitignore` and pre-commit hooks for you.
- `kerblam data` fetches remote data and saves it locally, manages local
  data and can clean it up, preserving only files that must be preserved.
  It also shows you how much local data is on the disk, how much data is
  remote and how much disk space you can free without losing anything important.
- `kerblam package` packages your pipeline and exports a `docker` image for
  execution later. It's useful for reproducibility purposes as the docker
  image is primed for execution, bundling the kerblam! executable, execution
  files and non-remote data in the blob itself.
- `kerblam run` executes the analysis for you, by choosing your `makefile`s and
  `dockerfiles` appropriately and building docker containers as needed.
  Optionally, allows test data or alternative data to be used instead of
  real data, in order to test your pipelines.
- `kerblam ignore` can edit your `.gitignore` file by adding files,
  folders and GitHub's recommended ignores for specific languages in just
  one command.
