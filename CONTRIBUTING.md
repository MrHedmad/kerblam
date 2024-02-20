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

## Testing
Tests are of two types, integration or unit tests. Both are run with `cargo test`.
Unit tests are your standard doc tests and unit tests, just write them in as
you'd normally would.

Kerblam! is also tested against [the kerblam-examples examples](https://github.com/MrHedmad/kerblam-examples).
Each example is in a directory (e.g. `examples/my_example`) with a `run` bash
script that runs the example and rolls it back.
`cargo test` checks out the *current* version of `kerblam-examples` and runs
the `run` scripts, checking:
- If the `run` script exited successfully;
- If the number or content of the files in the repo has changed from before
  the `run` script was executed.

To add a new example/test, write the example (see the `kerblam examples` repo),
add the `run` script to the example and add the `run_example_named!` or
`run_example_named!` macro with the name of the example and if you expect
it to succeed or fail, e.g. `run_example_named!("my_example", "success")`.

### Testing docker
Since CIs here on github are run in docker containers, there is a problem when
trying to test kerblam! on the cloud:

> The issue here is that when you run dockerized pipelines, the docker
> container is connected through a socket to the host container.
> Therefore, when we bind-mount paths inside the docker container, the bind
> fails since the host docker daemon does not "see" inside the runtime.
> It attempts to bind the local (on the host machine of Github) directory
> but it simply fails since it's not what we want to bind to, resulting
> in an empty mountpoint.
> 
> I think there is not easy solution here.

For this reason, all tests that run Docker at some point are wrapped in the
`run_ignored_example_named` macro instead. To run them locally, use
`cargo test -- --include-ignored`.

These example tests run locally on your machine, so you should make sure that you have
properly installed the requirements for kerblam! to run, including both docker
and podman executables.
If you find that one of these tests has failed, please be sure that it is not
due to your specific environment before starting the debug process.

## Contributions
Currently, the Kerblam! maintainer is [MrHedmad](https://github.com/MrHedmad).
There are no other maintainers. All contributions are listed below
by the @all-contributors bot.

Significant contributions will be recognized by a spot in the maintainers list,
at the discretion of other maintainers.

### All contributors

Thank you to all of these wonderful people!
Emojis reflect the specific contribution of that user.
See [the emoji key](https://allcontributors.org/docs/en/emoji-key)
(or you can hover with your mouse over each emoji to see its meaning).

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://mrhedmad.github.io/blog/"><img src="https://avatars.githubusercontent.com/u/46203625?v=4?s=100" width="100px;" alt="Luca "Hedmad" Visentin"/><br /><sub><b>Luca "Hedmad" Visentin</b></sub></a><br /><a href="#code-MrHedmad" title="Code">💻</a> <a href="#doc-MrHedmad" title="Documentation">📖</a> <a href="#ideas-MrHedmad" title="Ideas, Planning, & Feedback">🤔</a> <a href="#projectManagement-MrHedmad" title="Project Management">📆</a> <a href="#tutorial-MrHedmad" title="Tutorials">✅</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->
