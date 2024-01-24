# Tutorial
This section outlines making and working with a Kerblam! project.
It covers creation, execution and day-to-day tasks that Kerblam! can help with.

## Creating a new project - `kerblam new`
Go in a directory where you want to store the new project and run `kerblam new test-project`.
Kerblam! asks you some setup questions:
- If you want to use [Python](https://www.python.org/);
- If you want to use [R](https://www.r-project.org/);
- If you want to use [pre-commit](https://pre-commit.com/);
- If you have a Github account, and would like to setup the `origin` of your
  repository to Github.com.

Say 'yes' to all of these questions to follow along.
Kerblam! will:
- Make a new git repository,
- create the `kerblam.toml` file,
- create all the directories detailed above,
- make a `.pre-commit-config` file for you,
- create a `venv` environment, as well as the `requirements.txt` and `requirements-dev.txt`
  files (if you opted to use Python),
- and setup the `.gitignore` file with appropriate ignores;

> [!TIP]
> Kerblam! will **NOT** do an `Initial commit` for you!

You can now start writing code!
The rest of this tutorial outlines common tasks with which you can use `kerblam` for.

## Executing code - `kerblam run`
Kerblam can be used to manage how your project is executed, where and on
what input files.

Say that you have a script in `./src/calc_sum.py`. It takes an input `.csv` file,
processes it, and outputs a new `.csv` file, using `stdin` and `stdout`.

You have an `input.csv` file that you'd like to process with `calc_sum.py`.
You could write a shell script or a makefile with the command to run.
We'll refer to these scripts as "pipes".
Here's an example makefile pipe:

```makefile
./data/out/output.csv: ./data/in/input.csv ./src/calc_sum.py
    cat $< | ./src/calc_sum.py > $@
```

You'd generally place this file in the root of the repository and run `make`
to execute it. This is perfectly fine for projects with a relatively simple
structure and just one execution pipeline.

Imagine however that you have to change your pipeline to run two different
jobs which share a lot of code and input data but have slightly (or dramatically)
different execution.
You might modify your pipe to accept `if` statements, or perhaps write many of
them and run them separately.
In any case, having a single file that has the job of running all the different
pipelines is hard, adds complexity and makes managing the different execution
scripts harder than it needs to be.

Kerblam! manages your pipes for you.
You can write different makefiles and/or shell files for different types of
runs of your project and save them in `./src/pipes/`.
When you `kerblam run`, Kerblam! looks into that folder, finds (by name) the
makefiles that you've written, and brings them to the top level of the project
(e.g. `./`) for execution.

For instance, you could have written a `./src/pipes/process_csv.makefile` for
the previous step, and you could invoke it with `kerblam run process_csv`.
You could then write more makefiles or shell files for other tasks and run
them similarly.

Kerblam! looks for files ending in the `.makefile` extension for makefiles and 
`.sh` for shell files.

### Adding descriptions
If you execute `kerblam run` without specifying a pipe (or you try to run a 
pipe that does not exist), you will get a message like this:
```
Error: no runtime specified. Available runtimes:
    process_csv
    🐋 save_plots
    generate_metrics
```
The whale emoji (🐋) represents pipes that have an associated Docker container.

If you wish, you can add additional information to this list by writing a section
in the makefile/shellfile itself. Using the same example as above:
```makefile
#? Calculate the sums of the input metrics
#?
#? The script takes the input metrics, then calculates the row-wise sums.
#? These are important since the metrics refer to the calculation.

./data/out/output.csv: ./data/in/input.csv ./src/calc_sum.py
    cat $< | ./src/calc_sum.py > $@
```
If you add this block of lines starting with `#? `, Kerblam! will use them as
descriptions.
The first "block" of text (`#? ` lines not separated by an empty line) will be
the short descripition.
The rest of the blocks will be the long description.

Kerblam will parse *all* lines starting with `#? `, although it's preferrable
to only have a single contiguous description block in each file.

The output of `kerblam run` will now read:
```
Error: no runtime specified. Available runtimes:
    process_csv :: Calculate the sums of the input metrics
    🐋 save_plots
    generate_metrics
```
Context!

### Containerized execution
If Kerblam! finds a Dockerfile of the same name as one of your pipes in the
`./src/dockerfiles/` folder (e.g. `./src/dockerfiles/process_csv.dockerfile`),
it will:
- Copy the dockerfile to the top folder, as `Dockerfile`;
- Run `docker build --tag kerblam_runtime .` to build the container;
- Run `docker run --rm -it -v ./data:/data kerblam_runtime`.

If you have your docker container `COPY . .` and have `ENTRYPOINT make`
(or `ENTRYPOINT bash`), you can then effectively have Kerblam! run your projects
in docker environments, so you can tweak your dependencies and tooling
(which might be different than your dev environment) and execute in a protected,
reproducible environment.

> [!IMPORTANT]
> Kerblam! will build the docker environments without moving the dockerfiles.
> This means that you **have to write `.dockerignore` files in the `./src/dockerfiles`
> directory instead of the root of the repository**.
> 
> As an added bonus, you can write custom dockerignores for each of your
> docker containers as `pipe.dockerfile.dockerignore`.\
> See [docker's documentation for this feature](https://docs.docker.com/engine/reference/commandline/build/#file)
> and the related ['using a dockerignore' section](https://docs.docker.com/engine/reference/commandline/build/#use-a-dockerignore-file).
> 
> Neat!

You can write dockerfiles for both `make` and `sh` pipes.

### Specifying data to run on
By default, Kerblam! will use your `./data/in/` folder as-is when executing pipes.
If you want the same pipes to run on different sets of input data, Kerblam! can
temporarily swap out your real data with this 'substitute' data during execution.

For example, your `process_csv.makefile` requires an input `./data/in/input.csv` file.
However, you might want to run the same pipe on another, `different_input.csv` file.
You could copy and paste the first pipe, modify it on every file you wish to
run differently. However, you then have to maintain two essentially identical
pipelines, and you are prone to adding errors while you do so.
You can use `kerblam` to do the same, but in a declarative, easy way.

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
Under the hood, Kerblam! will:
- Rename `input.csv` to `input.csv.original`;
- Symlink `different_input.csv` to `input.csv`;
- Run the analysis as normal;
- When the run ends (or the analysis crashes), Kerblam! will remove the symlink
  and rename `<hex>_input.csv` back to `input.csv`.

This effectively causes the makefile run with different input data in this
alternate run.

> [!WARNING]
> Careful that the *output* data will (most likely) be saved as the
> same file names as a "normal" run! Kerblam! does not look into where the
> output files are saved or what they are saved as.

> [!WARNING]
> Careful! As of now, kerblam! has no problem overwriting existing
> files (e.g. `input.csv.original`) while running. See issue [#9](https://github.com/MrHedmad/kerblam/issues/9).

This is most commonly useful to run the pipelines on test data that is faster to
process or that produces pre-defined outputs. For example, you could define
something similar to:
```toml
[data.profiles.test]
"input.csv" = "test_input.csv"
"configs/config_file.yaml" = "configs/test_config_file.yaml"
```
And execute your test run with `kerblam run pipe --profile test`.

File paths specified under the `profiles` tab must be relative to the `./data/in/`
folder.

> [!TIP]
> Kerblam! tries its best to cleanup after itself (e.g. undo profiles,
> delete temporary files, etc...) when you use `kerblam run`, even if the pipe
> fails, and even if you kill your pipe with `CTRL-C`.

## Managing local and remote data
Kerblam! can help you retrieve remote data and manage your local data.

`kerblam data` will give you an overview of the status of local data:
```
> kerblam data
./data       500 KiB [2]
└── in       1.2 MiB [8]
└── out      823 KiB [2]
──────────────────────
Total        2.5 Mib [12]
└── cleanup  2.3 Mib [9] (92.0%)
└── remote   1.0 Mib [5]
! There are 3 undownloaded files.   
```
The first lines highlight the size (`500 KiB`) and amount (`2`) of files in the
`./data/in` (input), `./data/out` (output) and `./data` (intermediate) folders.

The total size of all the files in the `./data/` folder is then broken down
between categories: the `Total` data size, how much data can be removed with
`kerblam data clean` or `kerblam data pack`, and how many files are specified
to be downloaded but are not yet present locally.

You can manipulate your data with `kerblam data` in several ways.
In the following sections we explain every one of these ways.

### `kerblam data fetch` - Fetch remote data
If you define in `kerblam.toml` the section `data.remote` you can have
Kerblam! automatically fetch remote data for you:
```toml
[data.remote]
# This follows the form "url_to_download" = "save_as_file"
"https://raw.githubusercontent.com/MrHedmad/kerblam/main/README.md" = "some_readme.md"
```
When you run `kerblam data fetch`, Kerblam! will attempt to download `some_readme.md`
by following the URL you provided.
Most importantly, `some_readme.md` is treated as a file that is remotely available
and therefore locally expendable for the sake of saving disk size (see the
`data clean` and `data pack` commands).

You can specify any number of URLs and file names in `[data.remote]`, one for
each file that you wish to be downloaded.

> [!NOTE]
> The download directory for all fetched data is `./data/in`, so if you specify
> `some/nested/dir/file.txt`, kerblam! will save the file in
> `./data/in/some/nested/dir/file.txt`.

> [!CAUTION]
> If you write an absolute path (e.g. `/some_file.txt`) kerblam! will treat the
> path as it should treat it - by making the `/some_file.txt` in the root of
> the filesystem (and most likely failing to do so).
> See issue #19 for a discussion on this point.

### `kerblam data clean` - Free local disk space safely
If you want to cleanup your data (perhaps you have finished your work, and would
like to save some disk space), you can run `kerblam data clean`.
Kerblam! will remove:
- All temporary files in `./data/`;
- All output files in `./data/out`;
- All input files that can be downloaded remotely in `./data/in`.
This essentially only leaves input data that cannot be retrieved remotely on
disk.

Kerblam! will consider as "remotely available" files that are present in the
`data.remote` section of `kerblam.toml`.

### `kerblam data pack` - Package and export your local data
Say that you wish to send all your data folder to a colleague for inspection.
You can `tar -czvf exported_data.tar.gz ./data/` and send your whole data folder,
but you might want to only pick the output and non-remotely available inputs.

If you run `kerblam data pack` you can do just that.
Kerblam! will create a `exported_data.tar.gz` file and save it locally with the
non-remotely-available `.data/in` files and the files in `./data/out`.
You can also pass the `--cleanup` flag to also delete them after packing.

You can then share the data pack with others.

## `kerblam package` - Export an executable version of pipelines
The `kerblam package` command is one of the most useful features of Kerblam!
It allows you to package everything needed to execute a pipeline in a docker
container and export it for execution later.

For example, say that you have a `process` pipe that uses `make` to run, and 
requires both a remotely-downloaded `remote.txt` file and a local-only
`precious.txt` file.

> [!IMPORTANT]
> You must have a dockerfile `process.dockerfile` for every pipeline
> that you want to package!

If you execute
```bash
kerblam package process --name my_process_package
```
Kerblam! will:
- Clone the `process` pipe to the root of the project;
- Clone the `process.dockerfile` to the root of the project;
- Edit the `Dockerfile`:
  - Copy the non-remote input files to `/data/in` (i.e. the `precious.txt` file);
  - Copy the Kerblam! executable to the root of the dockerfile;
  - Configure the default command to `kerblam fetch && make .`
- Build the docker container and tag it with `my_process_package`;

> [!TIP]
> If you don't specify a `--name`, Kerblam! will name the result as `<pipe>_exec`.
> The `--name` parameter is a docker tag. You can specify a remote repository
> and push it with `docker push ...` as you would normally do.

After Kerblam! packages your project, you can re-run the analysis with:
```bash
docker run --rm -it -v /some/output/dir:/data/in my_process_package
```
In the container, Kerblam! fetches remote files and then the pipeline is triggered.
Since the output folder is attached to the output directory on disk, the 
final output of the pipeline is saved locally.

These packages are meant to make pipelines reproducible in the long-term.
For day-to-day runs, `kerblam run` is still better.

> [!CAUTION]
> The responsibility of having the resulting docker work in the long-term is
> up to you, not Kerblam!
> For most cases, just having `kerblam run` work is enough for the resulting
> package made by `kerblam package` to work, but depending on your docker
> files this might not be the case.
>
> For example, ubuntu docker containers may need to install `ca-certificates`
> to allow `kerblam` to fetch the remote data from inside the container.
>
> Kerblam! does not test the resulting package - it's up to you to do that.

> [!TIP]
> Even a broken `kerblam package` is still useful!
> You can always enter with `--entrypoint bash` and interactively work inside the
> container later, manually fixing any issues that time or wrong setup might
> have introduced.

## `kerblam ignore` - Add items to your `.gitignore` quickly
Oops! You forgot to include your preferred language to your `.gitignore`.
You now need to google for the template `.gitignore`, open the file and copy-paste it in.

With Kerblam! you can do that in just one command. For example:
```bash
kerblam ignore Rust
```
will fetch `Rust.gitignore` from the [Github gitignore repository](https://github.com/github/gitignore)
and append it to your `.gitignore` for you.
You can also add specific files or folders this way:
```bash
kerblam ignore ./src/something_useless.txt
```
Kerblam! will add the proper pattern to the `.gitignore` file to filter out
that specific file.

The optional `--compress` flag makes Kerblam! check the `.gitignore` file for
duplicated entries, and only retain one copy of each pattern.
This also cleans up comments and whitespace in a sensible way.

> [!TIP]
> The `--compress` flag allows to fix ignoring stuff twice.
> E.g. `kerblam ignore Rust && kerblam ignore Rust --compress` is the same as
> running `kerblam ignore Rust` just once.
