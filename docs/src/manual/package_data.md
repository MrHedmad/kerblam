# Package and distribute data
Say that you wish to send all your data folder to a colleague for inspection.
You can `tar -czvf exported_data.tar.gz ./data/` and send your whole data folder,
but you might want to only pick the output and non-remotely available inputs,
and leave re-downloading the (potentially bulky) remote data to your colleague.

> [!FAILURE]
> It is widely known that remembering `tar` commands is impossible.

If you run `kerblam data pack` you can do just that.
Kerblam! will create a `exported_data.tar.gz` file and save it locally with the
non-remotely-available `.data/in` files and the files in `./data/out`.
You can also pass the `--cleanup` flag to also delete them after packing.

You can then share the data pack with others.
