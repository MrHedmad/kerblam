# Quickstart

Welcome to Kerblam! This page will give you a hands-on introduction.
If you like what you see, you can check out [the manual](intro.md)
to learn all there is to know about Kerblam!

To follow along, be sure to be comfortable in the command line and
[install Kerblam!](install.md), as well as `wget` and Python.
Have access to a text editor of some sort.
If you want to follow along for the conainerization section, also install
Docker and be sure you can run it.

For this test project we will use Python to process some toy input data and
make a simple plot. We will create a simple `make` workflow to handle the
execution, and showcase all of Kerblam! features.

## Making a new project
Move to a location where the new project will be stored. Run:
```sh
kerblam new example_project
```
Go in a directory where you want to store the new project and run `kerblam new test-project`.
Kerblam! asks you some setup questions:
- If you want to use [Python](https://www.python.org/);
- If you want to use [R](https://www.r-project.org/);
- If you want to use [pre-commit](https://pre-commit.com/);
- If you have a Github account, and would like to setup the `origin` of your
  repository to [github.com](https://github.com).

Say 'yes' to all of these questions to follow along. Kerblam! will then:
- Create the project directory,
- initialise it as new git repository,
- create the `kerblam.toml` file,
- create all the default project directories,
- make an empty `.pre-commit-config` file for you,
- create a `venv` environment, as well as the `requirements.txt` and `requirements-dev.txt`
  files (if you opted to use Python),
- and setup the `.gitignore` file with appropriate ignores.

You can now start working in your new project, simply `cd test-project`.
Since Kerblam took care of making a virtual environment, use `source env/bin/activate`
to start working in it.

Take a moment to see the structure of the project. Note the `kerblam.toml` file,
that marks this project as a Kerblam! project (akin to the `.git` folder for `git`).

> [!TIP]
> You could use `tree .` to do this.
> See [the tree utility](https://en.wikipedia.org/wiki/Tree_(command)).

## Get input data
The input data we will use is available online
[in this gist](https://gist.github.com/MrHedmad/261fa39cd1402eaf222e5c1cdef18b3e).
It is the famous [Iris data](https://en.wikipedia.org/wiki/Iris_flower_data_set)
from [Fisher, R. A. (1936),
"The use of multiple measurements in taxonomic problems.", Annals of Eugenics,
7, Part II, 179–188.](https://doi.org/10.1111/j.1469-1809.1936.tb02137.x.), as
reported by R's `data(iris)` command.

We can use Kerblam! to [fetch input data](manual/fetch_data.md).
Open the `kerblam.toml` file and add at the bottom:
```toml
[data.remote]
"https://gist.githubusercontent.com/MrHedmad/261fa39cd1402eaf222e5c1cdef18b3e/raw/0c2ad0228a1d7e7b6f01268e4ee2ee01a55c9717/iris.csv" = "iris.csv"
"https://gist.githubusercontent.com/MrHedmad/261fa39cd1402eaf222e5c1cdef18b3e/raw/0c2ad0228a1d7e7b6f01268e4ee2ee01a55c9717/test_iris.csv" = "test_iris.csv"
```

> [!NOTE]
> The benefit of letting Kerblam! handle data retrieval for you is that, later,
> it can delete this remote data to save disk space.

Save the file and run
```bash
kerblam data fetch
```

Kerblam! will fetch the data and save it in `data/in`. You can check how your
disk is being used by using `kerblam data`. You'll see a summary like this:
```bash
>> kerblam data
./data	0 B [0]
└── in	4 KiB [2]
└── out	0 B [0]
──────────────────────
Total	4 KiB [2]
└── cleanup	4 KiB [2] (100.00%)
└── remote	4 KiB [2]
```

## Write the processing logic

We will take the input Iris data and make a simple plot.
Kerblam! has already set up your repository to use [the `src/` folder](https://stackoverflow.com/questions/23730882/what-is-the-role-of-src-and-dist-folders),
so we can start writing code in it.

Save this Python script in `src/process_csv.py`:
```python
import pandas as pd
import matplotlib.pyplot as plt
import sys

print(f"Reading data from {sys.argv[1]}")

iris = pd.read_csv(sys.argv[1])
species = iris['Species'].unique()
colors = ['blue', 'orange', 'green']

plt.figure(figsize=(14, 6))

for spec, color in zip(species, colors):
    subset = iris[iris['Species'] == spec]
    plt.hist(subset['Petal.Length'], bins=20, alpha=0.6, label=f'{spec} Petal Length', color=color, edgecolor='black')

for spec, color in zip(species, colors):
    subset = iris[iris['Species'] == spec]
    plt.hist(subset['Petal.Width'], bins=20, alpha=0.6, label=f'{spec} Petal Width', color=color, edgecolor='black', hatch='/')

plt.title('Distribution of Petal Length and Width')
plt.xlabel('Size (cm)')
plt.ylabel('Frequency')
plt.legend(title='Species and Measurement', loc='upper right')

plt.tight_layout()

print(f"Saving plot to {sys.argv[2]}")

plt.savefig(sys.argv[2], dpi = 300)
```

Since we are in a Python virtual environment, we can use `pip install pandas matplotlib`
to install the required packages for this script.

We can use the python script with a command like this:
```bash
python src/process_csv.py data/in/iris.csv data/out/plot.png
```

Try it now! It should create a `plot.png` file which you can manually inspect.

## Create and run a workflow
This is a very simple example, but a lot of day-to-day data analysis is
relatively straightforward.
In this case, we do not need a rich workflow manager: a bash script does the trick.

We can let Kerblam! handle the execution through Bash.
Create the `/src/workflows/create_iris_plot.sh` file and write in the command
from above:
```bash
python src/process_csv.py data/in/iris.csv data/out/plot.png
```

Now try it out! Run `kerblam run create_iris_plot` and see what happens.
Kerblam! has handled moving your workflow to the top-level of the project
(else the command would not work - it uses relative paths!) and executed
`bash` to run the command.

## Swap input files
We also downloaded a `test_iris.csv` dataset.
We might want to also use it to create the same plot.
We could edit the `create_iris_plot` to change the input file, or maybe
copy-and-paste it into a new `create_test_iris_plot`, but it would be verbose,
tedious and error-prone.

Instead, we can use Kerblam [profiles](manual/run.html#data-profiles---running-the-same-workflows-on-different-data)
that do this for us.
The `test` profile requires no configuration, so go right ahead and run
```bash
kerblam run create_iris_plot --profile test
```
and see how the `plot.png` file changes (the test data has less entries, so
the plot should be less dense).

## Run in a container

> [!NOTE]
> To run the examples in this section, you must have Docker installed and
> configured.

Our analysis is complete, but it's not reproducible with a simple bash script.
Kerblam! helps in this: we can run everything in a docker container pretty
easily.

Create the `src/dockerfiles/create_iris_plot.dockerfile` file, and write in:

```dockerfile
FROM python:latest

RUN pip install pandas matplotlib

COPY . .
```

There is no need to reference the actual workflow in the dockerfile.
Kerblam! takes care of everything for you.

To reduce the size of the image, it's a good idea to create the `.dockerignore`
file in the root of the project (next to `kerblam.toml`).
We can exclude the `data` and `env` folder from the container safely:
```dockeignore
data
env
```

Now we can run again:
```bash
kerblam run create_iris_plot
```
Kerblam! picks up automatically that a dockerfile is present for this workflow,
builds the image and uses it as a runtime.

> [!TIP]
> You can use profiles even with docker containers!

## Packaging data
We're done, and we'd like to share our work with others.
The simplest is to send them the output.
Kerblam! can help: run the following:
```bash
kerblam data pack
```
Kerblam! creates the `data/data_export.tar.gz` file with all the output data
of your project (and eventually input data that cannot be downloaded from
the internet, but this example does not have any).
You can share this tarball with colleagues quite easily.

## Packaging execution
If you used a container, you can also have Kerblam! package the workflow
proper for you.
Just run:
```bash
kerblam package create_iris_plot --tag my_test_container
```

Kerblam! will create the `my_test_container` image (so you can upload it to
the registry of your choice) and a tarball with the necessary input data plus
other bits and bobs that are needed to replay your run: the *replay package*.

Speaking of replay, you can do just that from the tarball you just created by
using `kerblam replay`:
```bash
# Let's move to an empty directory
mkdir -p test-replay && cd test-replay
kerblam replay ../
```
Kerblam! unpacks the tarball for you, creates a dummy project directory,
fetches the remote input data and runs the pipeline for you in the correct
docker container, automatically.

You can use replay packages manually too - they include the kerblam binary
that created them, so the reproducer does not need to leave anything to chance.

## Cleaning up
We're done! The output is sent to the reviewers, together with the replay
package, and we can close up shop.

If you don't want to completely delete the project, you can make it lightweight
by using Kerblam!.

Run:
```bash
kerblam data clean
```
Kerblam! will clean out all output data, intermediate (in `data/`) data and
input data that can be fetched remotely, saving you disk space for dormant
projects.

## Conclusions

Hopefully this toy example got you excited to use Kerblam!.
It only showcases some of Kerblam! features.
Read [the manual](manual/intro.md) to learn all theres is to know about how
Kerblam! can make your life easier.

