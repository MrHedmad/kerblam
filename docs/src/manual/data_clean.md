# Cleanup data
If you want to cleanup your data (perhaps you have finished your work, and would
like to save some disk space), you can run `kerblam data clean`.

Kerblam! will remove:
- All temporary files in `./data/`;
- All output files in `./data/out`;
- All input files that can be downloaded remotely in `./data/in`.
- All empty (even nested) folders in `./data/` and `./data/out`.
This essentially only leaves input data that cannot be retrieved remotely on
disk.

Kerblam! will consider as "remotely available" files that are present in the
`data.remote` section of `kerblam.toml`.
See [this chapter of the book](fetch_data.html) to learn more about remote data.
If you wish to preserve the remote data (perhaps you merely want to "reset"
the pipelines but start again quickly) you can use the `--keep-remote` flag to do so.

If you want to preserve the empty folders left behind after cleaning,
pass the `--keep-dirs` flag to do just that.

Kerblam! will ask for your confirmation before deleting the files.
If you're feeling bold, skip it with the `--yes` flag.
