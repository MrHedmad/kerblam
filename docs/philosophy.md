# The Kerblam! philosophy

> [!NOTE]
> Hello! This is the maintainer.
> This article covers the design principles behind how Kerblam! functions.
> It is both targeted at myself - to remind me why I did what I did - and to 
> anyone who is interested in the topic of managing data analysis projects.
>
> Reading this is not at all necessary to start using Kerblam!.
> Perhaps you want to [read the tutorial](docs/tutorial.md) instead.

> [!WARNING]
> I am an advocate of [open science](https://en.wikipedia.org/wiki/Open_science),
> [open software](https://en.wikipedia.org/wiki/Open-source_software) and of
> sharing your work as soon and as openly as possible.
> I also believe that documenting your code is even more important than the code itself.
>
> I make excessive use of these text boxes as I'm terribly bad at not including asides.
> Sorry!
> 
> Keep this in mind when reading this article, as it is strongly opinionated.

> [!TIP]
> The first time I use an acronym I'll make it ***bold italics*** so you can
> have an easier time finding it if you forget what it means.
> 
> I try to keep acronyms to a minimum.

## Introduction
After three years doing bioinformatics work as my actual job, I think I have
come across many of the different types of projects that one encounters as a bioinformatician:
1. You need to analyse some data either directly from someone or from some
   online repository.
   This requires the usage of both pre-established tools and new code and/or
   some configuration.
   - For example, someone in your research group performed RNA-Seq, and you
     are tasked with the data analysis.
2. You wish to create a new tool/pipeline/method of analysis and apply it to
   some data to both test its performance and/or functionality, before releasing
   the software package to the public.

The first point is **data analysis**. The second point is **software development**.
Both require writing software, but they are not exactly the same.

You'd generally work on point 3 like a generalist programmer would.
In terms of *how* you work, there are many different workflow mental schemas
that you can choose from, each with its following, pros, and cons.
Simply [search for coding workflow](https://duckduckgo.com/?t=h_&q=coding+workflow)
to find a plethora of different styles, methods and types of way you can use
to manage what to do and when while you code.

In any case, while working with a specific programming language, you usually
have only one PL.
A python project uses a [quite specific structure](https://docs.python-guide.org/writing/structure/):
you have a `pyproject.toml`/`setup.py`, a module directory[^pyweird]...
Similarly, when you work on a Rust project, you use `cargo`, and therefore
have a `cargo.toml` file, a `/src` directory...

> [!NOTE]
> The topic of structuring the code itself is even deeper, with different ways to
> think of your coding problem: object oriented vs functional vs procedural, 
> monolithic vs microservices, etcetera, but it's out of the scope of this piece.

At its core, software is a collection of text files written in a way that the
computer can understand.
The process of laying out these files in a logical way in the filesystem is
what I refer to when I say ***project layout (PL)***.
A ***project layout system (PLS) *** is a pre-established way to layout these files.
Kerblam! is a tool that can help you with general tasks if you follow the
Kerblam! project layout system.

> [!TIP]
> There are also project *management* systems, that are tasked with managing
> what has to be done while writing code.
> They are not the subject of this piece, however.

Since we are talking about code, there are a few characteristics in common between
all code-centric projects:
- The changes between different versions of the text files are important.
  We need to be able to go back to a previous version if we need to.
  This can be due by a number of things:
  if we realize that we changed something that we shouldn't have,
  if we just want to see a previous version of the code or
  if we need to run a previous version of the program for reproducibility purposes.
- Code must be documented to be useful. While it is often sufficient to read a
  piece of code to understand what it does, the *why* is often unclear.
  This is even more important when creating new tools: a tool without clear
  documentation is unusable, and an unusable tool might as well not exist.
- Often, code has to be edited by multiple people simultaneously.
  It's important to have a way to coordinate between people as you add your
  edits in.
- Code layout is often driven by convention or by requirements of build systems/
  interpreters/external tools that need to read your code.
  Each language is unique under this point.

From these initial observations we can start to think about a generic PLS.
Version control takes care of - well - version control and is essential for collaboration.
Version control generally does not affect the PL meaningfully.
However, version control often does not work well with large files, especially
binary files.

> [!IMPORTANT]
> **Design principle A: We must use a version control system.**<br>
> **Design principle B: Big binary blobs bad[^proud]!**

[^proud]: I'm very proud of this pun. Please don't take it from me.

> [!NOTE]
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

Design principle A makes it so that the basic unit of our PLS is the [repository](https://en.wikipedia.org/wiki/Repository_(version_control)).
Our project therefore is a repository of code.

As we said, documentation is important.
It should be versioned together with the code, as that is what it is
describing and it should change at the same page.

> [!IMPORTANT]
> **Design principle C: Documentation is good. We should do more of that.**

Code is [read more times than it is written](https://www.goodreads.com/quotes/835238-indeed-the-ratio-of-time-spent-reading-versus-writing-is),
therefore, it's important for a PLS to be logical and obvious.
To be logical, one should categorize files based on their content, and logically
arrange them in a way that makes sense when you or a stranger looks through them.
To be obvious, the categorization and the choice of folder and file names should
make sense at a glance (e.g. the '`scripts`' directory is for scripts, not for data).

> [!IMPORTANT]
> **Design principle D: Be logical, obvious and predictable**

Scientific computing needs to be reproduced by others.
The best kind of reproducibility is [computational reproducibility](https://lakens.github.io/statistical_inferences/14-computationalreproducibility.html),
by which the same output is generated given the same input.
There are a lot of things that you can do while writing code to achieve
computational reproducibility, but one of the main contributors to reproducibility
is still containerization.

Additionally, being *easily* reproducible is - in my mind - as important to being
reproducible to begin with.
The easier it is to reproduce your work, the more "morally upright" you will be in the
eyes of the reader.
This has a lot of benefits, of course, with the main one being that you are more
resilient to backlash in the inevitable case that you commit an error.

> [!IMPORTANT]
> **Design principle E: Be (easily) reproducible.**

## Structuring data analysis
While structuring single programs is relatively straightforward, doing the
same for a data analysis project is less set in stone.
However, given the design principles that we have established in the previous
section, we can try to find a way to fulfill all of them for the broadest
scope of application possible.

To design such a system, it's important to find what are the points in common
between all types of data analysis projects.
In essence, a data analysis project encompasses:
- Input data that must be analysed in order to answer some question.
- Output data that is created as a result of analysing the input data.
- Code that analyses that data.
- It is often the case that data analysis requires many different external tools,
  each with its own set of requirements. These sum with the requirements of your
  own code and scripts.

"Data analysis" code is not "tool" code: it usually uses more than one programming
language, it is not monolithic (i.e builds up to just one "thing") and can
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
The kerblam! way to structure a project is based on the design principles 
that we have seen, the characteristics of all data analysis project and some
additional fundamental observations, which I list below:
1. All projects deal with input and output data.
2. Some projects have intermediate data that can be stored to speed up the
   execution, but can be regenerated if lost (or the pipeline changes).
3. Some projects generate temporary data that is needed during the pipeline
   but then becomes obsolete when the execution ends.
4. Projects may deal with very large data files.
5. Projects may use different programming languages.
6. Projects, especially exploratory data analysis, require a record of all
   the trials that were made during the exploratory phase.
   Often, one last execution is the final one, with the resulting output the
   presented one.
Having these in mind, we can start to outline how Kerblam! deals with each of them.

### Data
Points 1, 2, 3 and 4 deal with data.
A kerblam! project has a dedicated `data` directory, as you'd expect.
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

To make the distinction between these different data types we could either keep
a separate configuration that points at each file (a git-like system),
or we specify directories where each type of file will be stored.

Kerblam! takes both of these approaches.
The distinction between input/output/temp/intermediate data is given by directories.
It's up to the user to save each file in the appropriate directory.
The distinction between remote and local files is however given by a config file,
`kerblam.toml`, so that Kerblam! can fetch the remote files for you on demand[^birds].
Fragile and precious data can just be computed from knowing the other two variables.

[^birds]: Two birds with one stone, or so they say.

The only data that needs to be manually shared with others is precious data.
Everything else can be downloaded or regenerated by the code.
This means that the only data that needs to be committed to version control
is the precious one.
If you strive to keep precious data to a minimum - as should already be the case -
analysis code can be kept tiny, size-wise.
This makes Kerblam! compliant with principle B[^pb] and makes it easier
(or in some cases possible) to be compliant with principle A[^pa].

### Execution
Points 5 and 6 are generally covered by pipeline managers.
A pipeline manager, like [snakemake](https://snakemake.readthedocs.io/en/stable/) 
or [nextflow](https://www.nextflow.io/), executes code in a controlled way
in order to obtain output files.
While both of these were made with data analysis in mind, they are both very
powerful and very "complex"[^notreally] and unwieldy for most projects.

Kerblam! supports simple shell scripts (which in theory can be used to run
anything, even pipeline managers like nextflow or snakemake) and makefiles natively.
`make` is a quite old GNU utility that is mainly used to build packages and
create compiled C/C++ projects.
However, it supports and manages the creation of any file with any creation recipe.
It is easy to learn and quick to write, and is at the perfect spot for most analyses
between a simple shell script and a full-fledged pipeline manager.

Kerblam! considers these executable scripts and makefiles as "pipes", where each
pipe can be executed to obtain some output.
Each pipe should call external tools and internal code.
If code is structured following the [`unix philosophy`](https://en.wikipedia.org/wiki/Unix_philosophy),
each different piece of code ("program") can be reused in the different pipelines and 
interlocked with one another inside pipelines.

With these considerations, point 6 can be addressed by making different pipes
with sensible names, saving them in version control.
Point 5 is easy if each program is independent of each other, and developed in
its own folder.
Kerblam! appoints the `./src` directory to contain the program code (e.g. scripts,
directories with programs, etc...) and the `/src/pipes` directory to contain shell
scripts and makefile pipelines.

These steps fulfill the design principle D[^pd]: Makefiles and shell scripts
are easy to read, and having separate folders for pipelines and actual code
that runs makes it easy to know what is what.
Having the rest of the code be sensibly managed is up to the programmer.

Principle E[^pe] can be messed up very easily, and the reproducibility crisis
is a symptom of this.
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
to be exactly the same as the output runs when development ends.

To be compliant with principle D[^pd], knowing what dockerfile is needed for what
pipeline can be challenging.
Kerblam! requires that pipes and the respective dockerfiles must have the same name.

### Documentation
Documentation is essential, as we said in principle C[^pc].
However, documentation is for humans, and is generally well established how to
layout the documentation files in a repository:
- Add a [`README`](https://en.wikipedia.org/wiki/README) file.
- Add a [`LICENSE`], so it's clear how other may use your code.
- Create a `/docs` folder with other documentation, such as `CONTRIBUTING` guides,
  tutorials and generally human-readable text needed to understand your project.

There is little that an automated tool can do to help with documentation.
There are plenty of guides online that deal with the task of documenting a project,
so I will not cover it further.

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

[^pb]: Big binary blobs bad.

[^pa]: We must use a version control system.

[^pd]: Be logical, obvious and predictable.

[^pe]: Be (easily) reproducible.

[^pc]: Documentation is good. We should do more of that.

---
