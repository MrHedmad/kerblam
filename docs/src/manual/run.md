# Running workflow managers - `kerblam run`

The `kerblam run` command is used to execute workflow managers for you.

Kerblam! looks for makefiles ending in the `.makefile` extension and 
`.sh` for shell files in the workflows directory (by default `src/pipes/`).
It automatically uses the proper execution strategy based on what extension
the file is saved as: either `make` or `bash`.

> [!IMPORTANT]
> Shell scripts are always executed in `bash`.

You can use any workflow manager that is installed on your system
through Kerblam! (e.g. `snakemake` or `nextflow`) by writing thin shell wrappers
with the execution command in the `src/pipes/` folder.
Make has a special execution policy to allow it to work with as little boilerplate
as possible.

`kerblam run` supports the following flags:
- `--profile <profile>`: Execute this workflow with a profile.
  Read more about profiles in the section below.
- `--desc` (`-d`): Show [the description of the workflow](workflow_docstrings.html), then exit.
- `--local` (`-l`): Skip [running in a container](run_containers.html), if a
  container is available, preferring a local run.

In short, `kerblam run` does something similar to this:
- Move your `workflow.sh` or `workflow.makefile` file in the root of the project,
  under the name `executor`;
- Launch `make -f executor` or `bash executor` for you.

This is why workflows are written as if they are executed in the root of the
project, because they are.

## Data Profiles - Running the same workflows on different data

You can run your same workflows, *as-is*, on different data thanks to data profiles.

By default, Kerblam! will leave `./data/in/` untouched when running workflow managers.
If you want the same workflows to run on different sets of input data, Kerblam! can
temporarily swap out your real data with this 'substitute' data during execution.

For example, a `process_csv.makefile` requires an input `./data/in/input.csv` file.
However, you might want to run the same workflow on another, `different_input.csv` file.
You could copy and paste the first workflows and change the paths to the first file
to this alternative one, or you might group variables into configuration
files for your workflow.
However, you then have to maintain two essentially identical workflows
(or several different configuration files),
and you are prone to adding errors while you modify them (what if you
forget to change one reference to the original file?).

You can let Kerblam! handle temporarely swapping input files for you,
without touching your workflows.
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
> [!TIP]
> Profiles work on directories too! If you specify a directory as a target
> of a profile, Kerblam! will move the whole directory to the new location.

> [!IMPORTANT]
> Paths under every profile section are relative to the input data directory,
> by default `data/in`.

Under the hood, Kerblam! will:
- Rename `input.csv` to `input.csv.original`;
- Move `different_input.csv` to `input.csv`;
- Run the analysis as normal;
- When the run ends (it finishes, it crashes or you kill it), Kerblam! will undo both actions:
  it moves `different_input.csv` back to its original place and
  renames `input.csv.original` back to `input.csv`.

This effectively causes the workflow to run with different input data.

> [!WARNING]
> Careful that the *output* data will (most likely) be saved as the
> same file names as a "normal" run!
> 
> Kerblam! does not look into where the output files are saved or what they
> are saved as.
> If you really want to, use the `KERBLAM_PROFILE` environment variable
> described below and change the output paths accordingly.

Profiles are most commonly useful to run the workflows on test data that is faster to
process or that produces pre-defined outputs. For example, you could define
something similar to:
```toml
[data.profiles.test]
"input.csv" = "test_input.csv"
"configs/config_file.yaml" = "configs/test_config_file.yaml"
```
And execute your test run with `kerblam run workflow --profile test`.

The profiles feature is used so commonly for test data that Kerblam! will
automatically make a `test` profile for you, swapping all input files in the
`./data/in` folder that start with `test_xxx` with their "regular" counterparts `xxx`.
For example, the profile above is redundant!

If you write a `[data.profiles.test]` profile yourself, Kerblam! will not
modify it in any way, effectively disabling the automatic test profile feature.

Kerblam! tries its best to cleanup after itself (e.g. undo profiles,
delete temporary files, etc...) when you use `kerblam run`, even if the workflow
fails, and even if you kill your workflow with `CTRL-C`.

> [!TIP]
> If your workflow is unresponsive to a `CTRL-C`, pressing it twice (two
> `SIGTERM` signals in a row) will kill Kerblam! instead, leaving the child
> process to be cleaned up by the OS and the (eventual) profile not cleaned up.
>
> This is to allow you to stop whatever Kerblam! or the workflow is doing in
> case of emergency.

### Detecting if you are in a profiled run

Kerblam! will run the workflows with the environment variable `KERBLAM_PROFILE`
set to whatever the name of the profile is.
In this way, you can detect from inside the workflow if you are in a profile or not.
This is useful if you want to keep the outputs of different profiles separate,
for instance.

### File modification times when using profiles
`make` tracks file creation times to determine if it has to re-run workflows again.
This means that if you move files around, like Kerblam! does when it applies
profiles, `make` will always re-run your workflows, even if you run the same
workflow with the same profile back-to-back.

To avoid this, Kerblam! will keep track of the last-run profile in your
projects and will update the timestamps of the moved files
*only when strictly necessary*.

This means that the profile files will get updated timestamps only when they
actually need to be updated, which is:
- When you use a profile for the first time;
- When you switch from one profile to a different one;
- When you don't use a profile, but you just used one the previous run;

To track what was the last profile used, Kerblam! creates a file in
`$HOME/.cache/kerblam/` for each of your projects.

### Sending additional arguments to the worker process
You can send additional arguments to either `make` or `bash` after what
Kerblam! sets by default by specifying them after kerblam's own `run` arguments:
```bash
kerblam run my_workflow -- extra_arg1 extra_arg_2 ...
```
Everything after the `--` will be passed as-is to the `make` or `bash`
worker after Kerblam!'s own arguments.

For example, you can tell `make` to build a different target with this syntax:
```bash
kerblam run make_workflow -- other_target
```
As if you had run `make other_target` yourself.

