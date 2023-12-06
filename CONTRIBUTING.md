# Contributing

Thank you for wanting to contribute to kerblam!

We use issues to track what needs to be done.
If you'd like to contribute, pick an issue, comment that you'd like to work
on it, and send a pull request when you are done.

Please keep all conversations civil, and do not discriminate anyone for their
experience, race or ethnicity, social status, etc.

## About the code
Kerblam! is relatively simple. Start reading the `main.rs` file and follow
the imports.
Each command is split in its own file.
Simple commands are all collected in the `other` module, since they most often
only encompass a function or two.

Kerblam! uses [`anyhow`](https://crates.io/crates/anyhow) to handle its `Result`s,
as most of the times we just want to end early, without special handling.

Any complex parts of the code should be commented and propely explained.
If you are unsure on what something does, and cannot decypher the code, please
open an issue - the code probably needs some refactoring if you feel that way.

## Contributions
Currently, the Kerblam! maintainer is [MrHedmad](https://github.com/MrHedmad).
There are no other maintainers. All contributions are listed below
by the @all-contributors bot.

Significant contributions will be recognized by a spot in the maintainers list,
at the discretion of other maintainers.

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->
