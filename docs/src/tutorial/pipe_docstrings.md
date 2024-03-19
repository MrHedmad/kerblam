# Describing pipelines
If you execute `kerblam run` without specifying a pipe (or you try to run a 
pipe that does not exist), you will get a message like this:
```
Error: no runtime specified. Available runtimes:
    ◾◾ process_csv
    🐋◾ save_plots
    ◾◾ generate_metrics
```
The whale emoji (🐋) represents pipes that [have an associated Docker container](run_containers.html).

If you wish, you can add additional information to this list by writing a section
in the makefile/shellfile itself. Using the same example as above:
```makefile
#? Calculate the sums of the input metrics
#?
#? The script takes the input metrics, then calculates the row-wise sums.
#? These are useful since we can refer to this calculation later.

./data/out/output.csv: ./data/in/input.csv ./src/calc_sum.py
    cat $< | ./src/calc_sum.py > $@
```
If you add this block of lines starting with `#? `, Kerblam! will use them as
descriptions (note that the space after the `?` is important!), and it will
treat them as [markdown](https://www.markdownguide.org/).
The first paragraph of text (`#? ` lines not separated by an empty `#?` line) will be
the title of the pipeline. Try to keep this short and to the point.
The rest of the lines will be the long description.

Kerblam will parse *all* lines starting with `#? `, although it's preferrable
to only have a single contiguous description block in each file.

The output of `kerblam run` will now read:
```
Error: no runtime specified. Available runtimes:
    ◾📜 process_csv :: Calculate the sums of the input metrics
    🐋◾ save_plots
    ◾◾ generate_metrics
```
The scroll (📜) emoji appears when Kerblam! notices a long description.
You can show the full description for such pipes with `kerblam run process_csv --desc`.

With pipeline docstrings, you can have a record of what the pipeline does for
both yourself and others who review your work.

You cannot write docstrings inside docker containers[^do_what_you_want].

[^do_what_you_want]: You actually can. I can't stop you. But Kerblam! ignores them.
