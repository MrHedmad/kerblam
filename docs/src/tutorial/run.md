# Executing code - `kerblam run`

The `kerblam run` command is used to run pipelines.

Kerblam! looks for files ending in the `.makefile` extension for makefiles and 
`.sh` for shell files in the pipelines directory (by default `src/pipes/`).
It automatically uses the proper execution strategy based on what extension
the file is saved as.

Shellfiles are *always executed in `bash`*. You can use anything that is
installed on your system this way, e.g. `snakemake` or `nextflow`.

Make has a special execution policy to allow it to work with as little boilerplate
as possible.
You can read more on Make [in the GNU Make book](https://www.gnu.org/software/make/manual/make.pdf).

`kerblam run` supports the following flags:
- `--profile <profile>`: Execute this pipeline with a profile.
  Read more about profiles in the section below.
- `--desc` (`-d`): Show [the description of the pipeline](pipe_docstrings.html), then exit.
- `--local` (`-l`): Skip [running in a container](run_containers.html), if a
  container is available, preferring a local run.

In short, `kerblam run` does something similar to this:
- Move your `pipe.sh` or `pipe.makefile` file in the root of the project,
  under the name `executor`;
- Launch `make -f executor` or `bash executor` for you.

This is why pipelines are written as if they are executed in the root of the
project, because they are.

## Data Profiles - Running the same pipelines on different data

You can run your same pipelines, *as-is*, on different data thanks to data profiles.

By default, Kerblam! will use your untouched `./data/in/` folder when executing pipes.
If you want the same pipes to run on different sets of input data, Kerblam! can
temporarily swap out your real data with this 'substitute' data during execution.

For example, a `process_csv.makefile` requires an input `./data/in/input.csv` file.
However, you might want to run the same pipe on another, `different_input.csv` file.
You could copy and paste the first pipe and change the paths to the first file
to this alternative one.
However, you then have to maintain two essentially identical
pipelines, and you are prone to adding errors while you modify it (what if you
forget to change one reference to the original file?).
You can use `kerblam` to do the same, but in an easy, declarative and less-error-prone way.

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

> Paths under every profile section are relative to the input data directory,
> by default `data/in`.

Under the hood, Kerblam! will:
- Rename `input.csv` to `input.csv.original`;
- Move `different_input.csv` to `input.csv`;
- Run the analysis as normal;
- When the run ends (it finishes, it crashes or you kill it), Kerblam! will undo both actions:
  it moves `different_input.csv` back to its original place and
  renames `input.csv.original` back to `input.csv`.

This effectively causes the makefile to run with different input data.

> Careful that the *output* data will (most likely) be saved as the
> same file names as a "normal" run!
> 
> Kerblam! does not look into where the output files are saved or what they
> are saved as.
> If you really want to, use the `KERBLAM_PROFILE` environment variable
> described below and change the output paths accordingly.

Profiles are most commonly useful to run the pipelines on test data that is faster to
process or that produces pre-defined outputs. For example, you could define
something similar to:
```toml
[data.profiles.test]
"input.csv" = "test_input.csv"
"configs/config_file.yaml" = "configs/test_config_file.yaml"
```
And execute your test run with `kerblam run pipe --profile test`.

The profiles feature is used so commonly for test data that Kerblam! will
automatically make a `test` profile for you, swapping all input files in the
`./data/in` folder that start with `test_xxx` with their "regular" counterparts `xxx`.
For example, the profile above is redundant!

If you write a `[data.profiles.test]` profile yourself, Kerblam! will not
modify it in any way, effectively disabling the automatic test profile feature.

Kerblam! tries its best to cleanup after itself (e.g. undo profiles,
delete temporary files, etc...) when you use `kerblam run`, even if the pipe
fails, and even if you kill your pipe with `CTRL-C`.

> If your pipeline is unresponsive to a `CTRL-C`, pressing it twice (two
> `SIGTERM` signals in a row) will kill Kerblam! instead, leaving the child
> process to be cleaned up by the OS and the (eventual) profile not cleaned up.
>
> This is to allow you to stop whatever Kerblam! or the pipe is doing in
> case of emergency.

Kerblam! will run the pipelines with the environment variable `KERBLAM_PROFILE`
set to whatever the name of the profile is.
In this way, you can detect from inside the pipeline if you are in a profile or not.
This is useful if you want to keep the outputs of different profiles separate,
for instance.

> Profiles work on directories too! If you specify a directory as a target
> of a profile, Kerblam! will move the whole directory to the new location.

### File modification times when using profiles
`make` tracks file creation times to determine if it has to re-run pipelines again.
This means that if you move files around, like Kerblam! does when it applies
profiles, `make` will always re-run your pipelines, even if you run the same
pipeline with the same profile back-to-back.

To avoid this, Kerblam! will keep track of the last-run profile in your
projects and update the timestamps of the moved files
**only when strictly necessary**.

This means that the profile files will get updated timestamps only when they
actually need to be updated, which is:
- When you use a profile for the first time;
- When you switch from one profile to a different one;
- When you don't use a profile, but you just used one the previous run;

To track what was the last profile used, Kerblam! creates a file in
`$HOME/.cache/kerblam/` for each of your projects.

## Sending additional arguments to make or bash
You can send additional arguments to either `make` or `bash` after what
Kerblam! sets by default by specifying them after kerblam's own `run` arguments:
```bash
kerblam run my_pipeline -- extra_arg1 extra_arg_2 ...
```
Everything after the `--` will be passed as-is to the `make` or `bash`
worker after Kerblam!'s own arguments.

For example, you can tell `make` to build a different target with this syntax:
```bash
kerblam run make_pipeline -- other_target
```
As if you had run `make other_target` yourself.
