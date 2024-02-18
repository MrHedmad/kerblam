# Fetching remote data
If you define in `kerblam.toml` the section `data.remote` you can have
Kerblam! automatically fetch remote data for you:
```toml
[data.remote]
# This follows the form "url_to_download" = "save_as_file"
"https://raw.githubusercontent.com/MrHedmad/kerblam/main/README.md" = "some_readme.md"
```
When you run `kerblam data fetch`, Kerblam! will attempt to download `some_readme.md`
by following the URL you provided and save it in the input data directory (e.g.
`data/in`).

Most importantly, **`some_readme.md` is treated as a file that is remotely available
and therefore locally expendable for the sake of saving disk size** (see the
[`data clean`](data_clean.html) and [`data pack`](package_data.html) commands).

You can specify any number of URLs and file names in `[data.remote]`, one for
each file that you wish to be downloaded.

The download directory for all fetched data is your input directory,
so if you specify `some/nested/dir/file.txt`, kerblam! will save the file in
`./data/in/some/nested/dir/file.txt`.
This also means that if you write an absolute path (e.g. `/some_file.txt`),
Kerblam! will treat the path as it should treat it - by making `some_file.txt`
in the root of the filesystem (and most likely failing to do so).
It will, however, warn you before acting that it is about to do something
potentially unwanted, giving you the chance to abort.
