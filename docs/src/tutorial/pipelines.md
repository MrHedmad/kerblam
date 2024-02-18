# Pipelines
Kerblam! is first and foremost a pipeline runner.

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
to execute it.

This is perfectly fine for projects with a relatively simple
structure and just one execution pipeline.

Imagine however that you have to change your pipeline to run two different
jobs which share a lot of code and input data but have slightly (or dramatically)
different execution.
You might modify your pipe to accept `if` statements, use environment variables
or perhaps write many of them and run them separately.
In any case, having a single file that has the job of running all the different
pipelines is hard, adds complexity and makes managing the different execution
scripts harder than it needs to be.

Kerblam! manages your pipes for you.
You can write different makefiles and/or shell files for different types of
runs of your project and save them in `./src/pipes/`.
When you `kerblam run`, Kerblam! looks into that folder, finds (by name) the
makefiles that you've written, and brings them to the top level of the project
(e.g. `./`) for execution.
In this way, you can write your pipelines *as if* they were in the root of
the repository, cutting down on a lot of boilerplate paths.

For instance, you could have written a `./src/pipes/process_csv.makefile` for
the previous step, and you could invoke it with `kerblam run process_csv`.
You could then write more makefiles or shell files for other tasks and run
them similarly, keeping them all neatly separated from the rest of the code.

The next sections outline the specifics of how Kerblam! executes pipes.
