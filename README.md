![If you want it, Kerblam it!](docs/images/logo.png)

> Most of this is not implemented yet.
> Consider this README a roadmap of sort of what kerblam! wants to be.

A Kerblam! project has a `kerblam.toml` file in its root.
If you use `kerblam`, you then get some nice perks:
- Allows for easy remote data access, by just specifying URLs and access tokens;
- Can package and export data quickly and easily to share the project with colleagues;
- Allows to manage and run multiple makefiles for different versions of the project;
- Leverages git to isolate, rollback and run the project at a different tag;
- Cleans up intermediate and output files quickly;
- Manages Docker environments and runs code in them for you.
- Manage the content of you `.gitignore` for you, allowing to add files, 
  directories and even whole languages in one command.
- Manages `.pre-commit-hooks` and Python `virtualenv`s for you.
- Allow you to specify test data to run

To transform a project to a Kerblam! project just make the kerblam.toml
file yourself.

## Overview
- `kerblam new` and `kerblam clone` can be used to create or clone a `kerblam` project.
  Kerblam! will ask you to fetch input files, create virtual environments and
  more upon creation.
- `kerblam fetch` fetches remote data and saves it locally.
- `kerblam clean` removes local data.
- `kerblam package` packages your pipeline and exports a `docker` image for
  execution later.
  Also can be used to export the data that the pipeline produces for sharing.
- `kerblam run` copies your project to a temporary directory, clones your
  `makefile`s and `dockerfiles` appropriately, builds a docker container and
  runs your analysis there.
  Optionally, allows test data to be used instead of real data, in order to
  test your pipelines.
- `kerblam ignore` can edit your `.gitignore` file by adding files, folders and
  GitHub's recommended ignores for specific languages in just one command.
- `kerblam link` can be used to move your `data` folder in some other place,
  and leave in its way a symlink, so that everything works just like before.
  This can be useful when your data is particularly bulky and you want to
  save it on some other drive.


## Opinions
Kerblam! projects are opinionated:
- The folder structure of your project adheres to the Kerblam! standard,
  although you may configure it in `kerblam.toml`.
  Read about it below.
- You use `make` or bash scripts as your pipeline manager.
- You use `docker` as your virtualization service.
- You use `git` as your version control system.
  Additionally, you create tags with `git` to record important previous 
  versions of your project.

If you don't like this setup, Kerblam! is not for you.

### Folder structure
Kerblam!, by default, requires the following folder structure (relative to the
root of the project, `./`):
- `./kerblam.toml`: This file contains the options for Kerblam!. It is usually empty.
- `./data/`: This is a directory for the data. Intermediate data files are held here.
- `./data/in/`: Input data files are saved and should be looked for, in here.
- `./data/out/`: Output data files are saved and should be looked for, in here.
- `./src/`: Code you want to be executed should be saved here.
- `./src/pipes/`: Makefiles and bash build scripts should be saved here.
  They have to be written as if they were saved in `./`.
- `./src/dockerfiles/`: Dockerfiles should be saved here. 
- `./env/`: Is the folder where the (eventual) python environment is located.
- `./requirements.txt`: Is the requirements file needed by `pip`;
- `./requirements-dev.txt`: Is the requirements file for development tools 
  needed by `pip`;

You can configure all of these paths in `kerblam.toml`, if you so desire.
This is mostly done for compatibility reasons.

## Contributing
Kerblam! is currently not accepting PRs as it's still in its infancy.

## Naming
This project is named after the fictitious online shop/delivery company in
[S11E07](https://en.wikipedia.org/wiki/Kerblam!) of Doctor Who.

Kerblam! might be referred to as Kerblam!, Kerblam or Kerb!am, interchangeably,
althoug Kerblam! is preferred.

## Installation
You can find and download a Kerblam! binary in
[the releases tab](https://github.com/mrhedmad/kerblam/releases).
Download it and drop it somewhere that you `$PATH` points to.

### Requirements
Kerblam! requires a linux (or generally unix-like) OS.
It also uses binaries that it assumes are already installed:
- GNU `make`: https://www.gnu.org/software/make/
- `git`: https://git-scm.com/
- Docker: https://www.docker.com/

If you can use `git`, `make` and `docker` from your CLI, you should be good.

# Tutorial
This section outlines making a Kerblam! project.

## Creating a new project
Go in a directory where you want to store the new project and run `kerblam new test-project`.
Kerblam! asks you some setup questions:
- If you want to use [Python](https://www.python.org/);
- If you want to use [R](https://www.r-project.org/);
- If you want to use [pre-commit](https://pre-commit.com/);
- If you have a github account, and would like to setup the `origin` of your
  repository to github.com.

Say 'yes' to all of these questions to follow along.
Kerblam! will:
- Make a new git repository,
- create the `kerblam.toml` file,
- create all the directories detailed above,
- make a `.pre-commit-config` file for you,
- create a `venv` environment, as well as the `requirements.txt` and `requirements-dev.txt`
  files,
- and setup the `.gitignore` file with appropriate ignores;

You can now start working! The rest of this tutorial outlines common tasks
with which you can use `kerblam` for.

## Executing code
Kerblam can be used to manage how your project is executed, where and on
what input files.

Say that you have a script in `./src/calc_sum.py`. It takes as input a `.csv`
and outputs a new `.csv` file as output, using `stdin` and `stdout`.
You have an `input.csv` file that you'd like to procees, to create an
`output.csv`.
You could write a shell script or a makefile with the command to run.
We'll refer to these scripts as "pipe"s.
Here's an example makefile:

```makefile
./data/out/output.csv: ./data/in/input.csv ./src/calc_sum.py
    cat $< | ./src/calc_sum.py > $@
```

This is great, until you have to run another "pipe", completely different from
the first one, with different steps, requiremets, etc...
You might write new makefiles or scripts for all the runs, but you'll then
have to remember how to structure each one, what each does and write a 
complex "meta"-makefile that runs the appropriate one.

Kerblam! manages your makefiles for you.
You can write different makefiles for different types of runs of your project,
and save them in `./src/pipes/`.
When you `kerblam run`, Kerblam! looks into that folder, finds (by name) the
makefiles that you've written, and brings them to the top level of the project
(e.g. `./`) for execution.

For instance, you could have written a `./src/pipes/process_csv.makefile` for
the previous step, and you could invoke it with `kerblam run process_csv`.
You could then write more makefiles for other tasks and run them similarly.

If Kerblam! finds a dockerfile of the same name as one of your pipes in the
`./src/dockerfiles/` folder (e.g. `./src/dockerfiles/process_csv.dockerfile`),
it will:
- Move the dockerfile to the top folder, next to the makefile;
- Run `docker build --tag <name_of_makefile> .` to build the container;
- Run `docker run --rm -it -v ./data:/data <name_of_makefile>`.

If you have your docker container `COPY . .` and have `ENTRYPOINT make`, you
can then effectively have Kerblam! run your projects in docker environments,
so you can tweak your dependencies and tooling (which might be different than
your dev environment).

Kerblam! will automatically `.dockerignore` your `./data/` folder (if it's not
the case already), since it's connected to the container at runtime instead.

The same applies to `.sh` files in the `./src/pipes/` directory.

---

And remember! If you want it...

![Kerblam it!](docs/images/kerblam_it.gif)

