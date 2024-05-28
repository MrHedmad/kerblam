# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-rc.3](https://github.com/MrHedmad/kerblam/compare/v1.0.0-rc.2...v1.0.0-rc.3) - 2024-05-28

### Other
- update cargo-dist
- add minimum rust version

## [1.0.0-rc.2](https://github.com/MrHedmad/kerblam/compare/v1.0.0-rc.1...v1.0.0-rc.2) - 2024-05-07

### Added
- add workflows community metadata and icons
- start using callouts in the book
- rename 'skip-build-cache' to 'no-build-cache'
- add flag to skip using container cache at build time
- cache is now a JSON for greater future extendability
- better (test) error reporting upon pipeline failure
- *(execution)* make cache path finding more robust
- *(execution)* properly manage timestamps of profile files

### Fixed
- calling type for docker with cache was wrong
- make properly changes working directory following kerblam.toml in docker

### Other
- *(deps)* bump rustls from 0.21.10 to 0.21.11
- add callout to block
- Merge branch 'main' into book-callouts
- remove wrong mention of extra arguments to build backend
- mention the 'skip-build-cache' flag
- extracted cache code as a module
- Merge branch 'main' into cache_profiles
- manually applied clippy suggestions
- apply clippy auto-fixes
- Merge branch 'main' into extra_args
- *(deps)* bump actions/configure-pages from 4 to 5
- fix landing image background
- miscellaneous book typos fix and clarifications
- update landing page figure
- modify landing page figure
- *(ci)* update release-plz to use bot account
- *(deps)* bump actions/deploy-pages from 1 to 4
- Merge branch 'main' into dependabot/github_actions/actions/upload-pages-artifact-3
- Merge branch 'main' into dependabot/github_actions/actions/configure-pages-4
- update cargo dist
- mention new CI in CONTRIBUTING.md
- trigger release-plz
- add release-plz
- add kerblam favicon
- force default dark + ayu theme
- *(deps)* bump mio from 0.8.10 to 0.8.11
- add pre-release installation warning to README
- Update README.md
