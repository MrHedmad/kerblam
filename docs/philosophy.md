# The Kerblam! philosophy

> ![NOTE]
> Hello! This is the maintainer.
> This is prose about why I chose to make Kerblam! like I did, both for
> myself to reference in the future and also because others have asked me why
> I chose the workflow that Kerblam! supports instead of any other.
>
> Reading this is not at all necessary to start using Kerblam!.
> Perhaps you want to [read the tutorial](docs/tutorial.md) instead.

> ![WARNING]
> I am an advocate of [open science](https://en.wikipedia.org/wiki/Open_science),
> [open software](https://en.wikipedia.org/wiki/Open-source_software) and of
> sharing your work as soon and as openly as possible.
> I also believe that documenting your code is even more important than the code itself.
> 
> Keep this in mind when reading this article, as it is strongly opinionated.

## Introduction
After three years doing bioinformatics work as my actual job, I think I have
come across many of the different types of projects that one encounters as a bioinformatician:
1. Someone asks you to analyse some data that they have created or found,
   with a specific question (and most often outcome) that they want answered.
2. You wish to analyse some data in order to discover something.
   Generally you did not create the data directly, but it is from some online
   repository.
3. You wish to create a new tool/pipeline/method of analysis and apply it to
   some data to both test its performance and/or functionality, before releasing
   the software package to the public.

Points 1 and 2 fall under the term "data analysis", while point 3 is more
software-development oriented.

> ![NOTE]
> I assume that the reader knows how vital version control is when writing software.
> In case that you do not, I want to briefly outline why you'd want to use a version
> control system in your work:
> - It takes care of tracking what you did on your project;
> - You can quickly turn back time if you mess up and change something that should
>   not have been changed.
> - It allows you to collaborate both in your own team (if any) and with the public
>   (in the case of open-source codebases).
>   Collaboration is nigh impossible without a version control system.
> - It allows you to categorize and compartimentalize your work, so you can keep
>   track of every different project neatly.
> - It makes the analysis (or tool) accessible - and if you are careful also 
>   reproducible - to others, which is an essential part of the scientific process.
> These are just some of the advantages you get when using a version control system.
> One of the most popular version control systems is `git`.
> With `git`, you can progressively add changes to code over time, with `git`
> taking care of recording what you did, and managing different versions made
> by others.
>
> If you are not familiar with version control systems and specifically with `git`,
> I suggest you stop reading and look up [the git user manual](https://git-scm.com/docs/user-manual).

### Structuring software
You'd generally work on point 3 like a generalist programmer would.
In terms of *how* you work, there are many different workflow mental schemas
that you can choose from, each with its following, pros, and cons.
Simply [search for coding workflow](https://duckduckgo.com/?t=h_&q=coding+workflow)
to find a plethora of different styles, methods and types of way you can use
to manage what to do and when while you code.

In any case, while working with a specific programming language, you usually
have only one way to *structure* your code.
A python project uses a [quite specific structure](https://docs.python-guide.org/writing/structure/):
you have a `pyproject.toml`/`setup.py`, a module directory, etc[^pyweird].
Similarly, when you work on a Rust project, you use `cargo`, and therefore
have a `cargo.toml` file, a `/src` directory, etc..

The topic of structuring the code itself is even deeper, with different ways to
think of your coding problem: object oriented vs functional vs procedural, 
monolithic vs microservices, etc...

### Structuring data analysis
Structuring your code for data analysis projects is a bit less defined.
Keep in mind that professional bioinformaticians need to follow some quality standards,
both as programmers (i.e. code quality) and as scientists (i.e. transparency).
Your code has to be functional (i.e. run), accessible and understandable by others.
If this is not the case, people need to trust that the one run that you did
to create the results included in your paper/report is correct.
Trust has no place in science.

For this reason, it helps to have a standard way to structure a data analysis project.
To design such a system, it's important to find what are the points in common
between all types of data analysis projects.
In essence, a data analysis project encompasses:
- Input data that must be analysed in order to answer some question.
- Output data that is created as a result of analysing the input data.
- Code that analyses that data.

"Data analysis" code is not "tool" code: it usually uses more than one programming
language, it is not monolythic (i.e builds up to just one "thing") and can
differ wildly in structure (from just one script, to external tool, to complex
pieces of code that run many steps of the analysis).

This complexity results in a plethora of different ways to structure the code
and the data during the project.
There are many examples of data analysis projects out in the wild, with wildly
different structures:
- // TODO: Add examples

I will not say that the Kerblam! way is the one-and-only, cover-all way to
structure your project, but I will say that it is a sensible default.

## Kerblam!
The kerblam! way to structure a project is based on some fundamental observations,
which I list below:
1. All projects deal with input and output data.
2. Some projects have intermediate data that can be stored to speed up the
   execution, but can be regenerated if lost (or the pipeline changes).
3. Some projects generate temporary data that is needed during the pipeline
   but then becomes obsolete when the execution ends.
4. Projects may deal with very large data files.
5. Projects need to be reproducible even after the original analysis ended.
   - This point can also be expanded to be "easily reproducible".
6. Projects may use different programming languages.
7. Projects, especially exploratory data analysis, require a record of all
   the trials that were made during the exploratory phase.

Having these in mind, we can start to outline how Kerblam! deal with each of them.
Points 1, 2, 3 and 4 deal with data. A kerblam! project has a dedicated `data`
directory, as you'd expect.
However, kerblam! actually differentiates between the different data types.
Other than input, output, temporary and intermediate data, kerblam! also considers:
- Remote data is data that can be downloaded at runtime from a (static) remote source.
- Input data that is not remote is called *precious*, since it cannot be
  substituted if it is lost.
- All data that is not precious is *fragile*, since it can be deleted with
  little repercussion (i.e. you just re-download it or re-run the pipeline to
  obtain it again.

> [!NOTE]
> Practically, data can be input/output/temp/intermediate, either fragile
> or precious and either local or remote.

To make this distinction we could either keep a separate configuration that
points at each file (a git-like system), or we specify directories where each
type of file will be stored.

Kerblam! takes both of these approaches.
The distinction between input/output/temp/intermediate data is given by directories.
It's up to the user to save each file in the appropriate directory.
The distinction between remote and local files is however given by a config file,
`kerblam.toml`, so that Kerblam! can fetch the remote files for you on demand.
Fragile and precious data can just be computed from knowing the other two variables.

The only data that needs to be manually shared with others is precious data.
Everything else can be downloaded or regenerated by the code.
This means that the only data that needs to be committed to version control
is the precious one.
If you strive to keep precious data to a minimum - as should already be the case -
analysis code can be kept tiny, size-wise.

Points 6 and 7 are generally covered by pipeline managers.
A pipeline manager, like [snakemak](https://snakemake.readthedocs.io/en/stable/) 
and [nextflow](https://www.nextflow.io/), executes code in a controlled way
in order to obtain output files.
While both of these were made with data analysis in mind, they are both very
powerful and very "complex"[^notreally] and unwieldy for most projects.

Kerblam! supports simple shell scripts (which in theory can be used to run
anything, even pipeline managers like nextflow or snakemake) and makefiles natively.
`make` is a quite old GNU utility that is mainly used to build packages and
create compiled C/C++ projects.
However, it supports and manages the creation of any file with any creation recipe.
It is easy to learn and quick to write, and is the perfect spot for most analyses
between a simple shell script and a full-fledged pipeline manager.

Kerblam! considers these executable scripts and makefiles as "pipes", where each
pipe can be executed to obtain some output.
Each pipe should call external tools and internal code.
If code is structured following the [`unix philosophy`](https://en.wikipedia.org/wiki/Unix_philosophy),
each different piece of code ("program") can be reused in the different pipelines and 
interlocked with one another inside pipelines.

With these considerations, point 7 can be addressed by making different pipes
with sensible names, saving them in version control.
Point 6 is easy if each program is independent of each other, and developed in
its own folder.
Kerblam! appoints the `./src` directory to contain the program code (e.g. scripts,
directories with programs, etc...) and the `/src/pipes` directory to contain shell
scripts and makefile pipelines.

Point 5 can be messed up very easily, and the reproducibiliy crisis is a sympthom of this.
A very common way to make any analysis reproducible is to package the execution
environment into containers, executable bundles that can be configured to do
basically anything in an isolated, controlled environment.

Kerblam! projects leverage docker containers to make the analysis as easily
reproducible as possible.
Using docker for the most basic tasks is relatively straightforward:
- Start with an image;
- Add dependencies;
- Copy the current environment;
- Setup the proper entrypoint;
- Execute the container with a directory mounted to the local file system in
  order to extract the output files as needed.

Kerblam! automatically detects dockerfiles in the `./src/dockerfiles` directory
and builds and executes the containers following this simple schema.
To give as much freedom to the user as possible, Kerblam! does not edit or check
these dockerfiles, just executes them in the proper environment and the correct
mounting points.

The output of a locally-run pipeline cannot be trusted as it is not reproducible.
Having Kerblam! natively run all pipelines in containers allows development runs
to be exactly the same as the output runs when the development ends.

---

[^pyweird]: Python packaging is a bit weird since there are so many packaging
engines that create python packages. Most online guides use `setuptools`, but
modern python (as of Dec 2023) now works with the `build` script with a 
`pyproject.toml` file, which supports different build engines.
See [this pep](https://peps.python.org/pep-0621/) for more info.
[^notreally]: I cannot find a good adjective other than "complex". These tools
are not hard to use, or particularly difficult to learn, but they do have an
initial learning curve. The thing that I want to highlight is that they are
so formal, and require careful specification of inputs, output, channels and 
pipelines that they become a bit unwieldy to use as a default. For large project
with many moving parts and a lot of computing (e.g. the need to run in a cluster),
using programs such as these can be very important and useful.
However, bringing a tank to a fist fight could be a bit too much.
