# Other utilities
Kerblam! has a few other utilities to deal with the most tedius steps when
working with projects.

## `kerblam ignore` - Add items to your `.gitignore` quickly
Oops! You forgot to include your preferred language to your `.gitignore`.
You now need to google for the template `.gitignore`, open the file and copy-paste it in.

With Kerblam! you can do that in just one command. For example:
```bash
kerblam ignore Rust
```
will fetch `Rust.gitignore` from the [Github gitignore repository](https://github.com/github/gitignore)
and append it to your `.gitignore` for you.
Be careful that this command is **case sensitive** (e.g. `Rust` works, `rust` does not).

You can also add specific files or folders this way:
```bash
kerblam ignore ./src/something_useless.txt
```
Kerblam! will add the proper pattern to the `.gitignore` file to filter out
that specific file.

The optional `--compress` flag makes Kerblam! check the `.gitignore` file for
duplicated entries, and only retain one copy of each pattern.
This also cleans up comments and whitespace in a sensible way.

The `--compress` flag allows to fix ignoring stuff twice.
E.g. `kerblam ignore Rust && kerblam ignore Rust --compress` is the same as
running `kerblam ignore Rust` just once.
