# Quickstart

Welcome to Kerblam!
This introductory chapter will give you the general overview on Kerblam!: what
it does and how it does it.

Kerblam! is a *project manager*. It helps you write clean, concise data analysis
pipelines, and takes care of chores for you.

Every Kerblam! project has a `kerblam.toml` file in its root.
When Kerblam! looks for files, it does it relative to the position of the
`kerblam.toml` file and in specific, pre-determined folders.
This helps you keep everything in its place, so that others that are unfamiliar
with your project can understand it if they ever need to look at it.

> [!TIP]
> Akin to `git`, Kerblam! will look in parent directories for a `kerblam.toml`
> file and run there if you call it from a project sub-folder.

These folders, relative to where the `kerblam.toml` file is, are:
- `./data/`: Where all the project's data is saved.
  Intermediate data files are specifically saved here.
- `./data/in/`: Input data files are saved and should be looked for in here.
- `./data/out/`: Output data files are saved and should be looked for in here.
- `./src/`: Code you want to be executed should be saved here.
- `./src/pipes/`: Makefiles and bash build scripts should be saved here.
  They have to be written as if they were saved in `./`.
- `./src/dockerfiles/`: Container build scripts should be saved here.

> [!TIP]
> Any sub-folder of one of these specific folders (with the exception of
> `src/pipes` and `src/dockerfiles`) contains the same type of files as the
> parent directory. For instance, `data/in/fastq` is treated as if it contains
> input data by Kerblam! just as the `data/in` directory is.

You can configure almost all of these paths in the `kerblam.toml` file, if you so desire.
This is mostly done for compatibility reasons with non-kerblam! projects.
New projects that wish to use Kerblam! are strongly encouraged to follow the
standard folder structure, however.

> [!IMPORTANT]
> The rest of these docs are written as if you are using the standard
> folder structure. If you are not, don't worry! All Kerblam! commands respect
> your choices in the `kerblam.toml` file.

If you want to convert an existing project to use Kerblam!, you can take a look
at [the `kerblam.toml` section of the documentation](kerblam.toml.html) to
learn how to configure these paths.

If you follow this standard (or you write proper configuration), you can use
Kerblam! to do a bunch of things:
- You can run pipelines written in `make` or arbitrary shell files in `src/pipes/`
  as if you ran them from the root directory of your project by simply using
  `kerblam run <pipe>`;
- You can wrap your pipelines in docker containers by just writing new
  dockerfiles in `src/dockerfiles`, with essentially just the installation
  of the dependencies, letting Kerblam! take care of the rest;
- If you have wrapped up pipelines, you can export them for later execution
  (or to send them to a reviewer) with `kerblam package <pipe>` without needing
  to edit your dockerfiles;
- If you have a package from someone else, you can run it with `kerblam replay`.
- You can fetch remote data from the internet with `kerblam data fetch`, see
  how much disk space your project's data is using with `kerblam data` and
  safely cleanup all the files that are not needed to re-run your project with
  `kerblam data clean`.
- You can show others your work by packing up the data with `kerblam data pack`
  and share the `.tar.gz` file around.
- And more!

The rest of this tutorial walks you through every feature.

I hope you enjoy Kerblam! and that it makes your projects easier to understand,
run and reproduce!

> [!INFO]
> If you like Kerblam!, please consider [leaving a star on Github](https://github.com/MrHedmad/kerblam/stargazers).
> Thank you for supporting Kerblam!
