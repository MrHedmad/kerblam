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

> [!DANGER]
> The download directory for all fetched data is your input directory,
> so if you specify `some/nested/dir/file.txt`, kerblam! will save the file in
> `./data/in/some/nested/dir/file.txt`.
> This also means that if you write an absolute path (e.g. `/some_file.txt`),
> Kerblam! will treat the path as it should treat it - by making `some_file.txt`
> in the root of the filesystem (and most likely failing to do so).
>
> Kerblam! will, however, warn you before acting, telling you that it is about
> to do something potentially unwanted, and giving you the chance to abort.

## Unfetcheable data
Sometimes, a simple GET request is not enough to fetch your data.
Perhaps you need some complicated login, or you use specific software to fetch
your remote data.
You can still tell Kerblam! that a file is remote, but that Kerblam! cannot
directly fetch it: this way you can use all other Kerblam! features but
"opt out" of the fetching one.

To do this, simply specify `"_"` as the remote URL in the `kerblam.toml` file:
```toml
[data.remote]
"https://example.com/" = "remote_file.txt"
"_" = "unfetcheable_file.txt"
```

If you run `kerblam data fetch` with the above command, you'll fetch the
`remote_file.txt`, but not `unfetcheable_file.txt` (and Kerblam! will remind
you of that).

> [!NOTE]
> Remember that [Kerblam! replay packages](./package_pipes.md) will fetch
> remote data for you before running the packaged workflow.
> If an unfetcheable file is needed by the packaged workflow, be sure to fetch
> it *inside* the workflow itself before running the computation proper.
