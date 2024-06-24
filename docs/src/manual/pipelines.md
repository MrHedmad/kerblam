# Managing multiple Workflows
Kerblam! can be used as a workflow manager *manager*.
It makes it easier to write multiple workflows for your project, keeping them
simple and ordered, and executing your workflow managers.

> [!INFO] What are workflows?
> When analysing data, you want to take an input file, apply some transformations
> on it (programmatically), and obtain some output.
> If this is done in one, small and easy step, you could run a single command
> on the command line and get it done.
> 
> For more complicated operations, where inputs and outputs "flow" into various
> programs for additional processing, you might want to describe the process of
> creating the output you want, and let the computer handle the execution itself.
> This is a [*workflow*](https://en.wikipedia.org/wiki/Workflow): a series of
> instructions that are executed to obtain some output.
> The program that reads the workflow and executes it is a *workflow manager*.
> 
> The simplest workflow manager is your shell: you can write a shell script
> with the commands that should be executed in order to obtain your output.
> 
> More feature-rich workflow managers exist. For instance, [make](https://www.gnu.org/software/make/)
> can be used to execute workflows[^make_workflows].
> The workflows written in make are called *makefiles*.
> You'd generally place this file in the root of the repository and run `make`
> to execute it.

When your project grows in complexity, and separate workflows emerge, they are
increasingly hard to work with.
Having a single file that has the job of running all the different
workflows that your project requires is hard, adds complexity and makes
running them harder than it needs to be.

**Kerblam! manages your workflows for you.**

Kerblam supports `make` out of the box, and all other workflow managers
through thin Bash wrappers.

You can write different makefiles and/or shell files for different types of
runs of your project and save them in `./src/pipes/`.
When you `kerblam run`, Kerblam! looks into that folder, finds (by name) the
workflows that you've written, and brings them to the top level of the project
(e.g. `./`) for execution.
In this way, you can write your workflows *as if* they were in the root of
the repository, cutting down on a lot of boilerplate paths.

For instance, you could write a `./src/pipes/process_csv.makefile`
and you could invoke it with `kerblam run process_csv`.

This lets you write separate workflows and keep your project compact,
non-redundant and less complex.

The next sections outline the specifics of how Kerblam! does this, as well
as other chores that you can let Kerblam! handle instead of doing them manually
yourself.

[^make_workflows]: Make is not a workflow manager per-se. It was created to
  handle the compilation of programs, where many different files have to be
  compiled and combined together. While workflows are not the reason that
  make was created for, it can be used to write them. In fact, an extended
  make-like workflow manager exists: [`makeflow`](https://cctools.readthedocs.io/en/latest/makeflow/).

