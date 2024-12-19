# Cleanup data
If you want to cleanup your data (perhaps you have finished your work, and would
like to save some disk space), you can run `kerblam data clean`.

Kerblam! will remove:
- All temporary files in `./data/`;
- All output files in `./data/out`;
- All empty (even nested) folders in `./data/` and `./data/out`.
This essentially only leaves input data on the dist.

To additionally clean remotely available data (to really put a project in
cold storage), pass the `--include-remote` flag.

Kerblam! will consider as "remotely available" files that are present in the
`data.remote` section of `kerblam.toml`.
See [this chapter of the book](fetch_data.html) to learn more about remote data.

If you want to preserve the empty folders left behind after cleaning,
pass the `--keep-dirs` flag to do just that.

Kerblam! will ask for your confirmation before deleting the files.
If you're feeling bold, skip it with the `--yes` flag.

With the `--preserve-output` flag, Kerblam! will skip deleting the output files.

## Dry runs
With the `--dry-run` option, Kerblam! will just show the list of files to be deleted,
without *actually* deleting anything:
```bash
> kerblam data clean --dry-run
Files to clean:
data/temp.csv
data/out/finala.txt
```
