# Installation
You have a few options when installing Kerblam!.

### Requirements
Currently, Kerblam! only supports mac OS (both intel and apple chips) and GNU linux.
Other unix/linux versions *may* work, but are untested.
It also uses binaries that it assumes are already installed and visible from your `$PATH`:
- GNU `make`: [gnu.org/software/make](https://gnu.org/software/make);
- `git`: [git-scm.com](https://git-scm.com/)
- Docker (as `docker`) and/or Podman (as `podman`):
  [docker.com](https://docker.com/) and/or [podman.io](https://podman.io);
- `bash`: [gnu.org/software/bash](https://www.gnu.org/software/bash/).

If you can use `git`, `make`, `bash` and `docker` or `podman` from your CLI,
you're good to go!

Most if not all of these tools come pre-packaged in most linux distros.
Check your repositories for them.

### Pre-compiled binary (recommended)
You can find and download a Kerblam! binary for your operating system in
[the releases tab](https://github.com/mrhedmad/kerblam/releases).

There are also helpful scripts that automatically download the correct version
for your specific operating system thanks to [`cargo-dist`](https://github.com/axodotdev/cargo-dist).
You can always install or update to the latest version with:
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MrHedmad/kerblam/releases/latest/download/kerblam-installer.sh | sh
```
Be warned that the above command executes a script downloaded from the internet.
You can [click here](https://github.com/MrHedmad/kerblam/releases/latest/download/kerblam-installer.sh)
or manually follow the fetched URL above to download the same installer script
and inspect it before you run it, if you'd like.

### Install from source
If you want to install the latest version from source, install Rust and `cargo`, then run:
```bash
cargo install kerblam
```
If you wish to instead use the latest development version, run:
```bash
cargo install --git https://github.com/MrHedmad/kerblam.git
```
The `main` branch should always compile on supported platforms with the above command.
If it does not, please [open an issue](https://github.com/mrhedmad/kerblam/issues/new).

## Adding the Kerblam! badge
You can add a Kerblam! badge in the README of your project to show that you use Kerblam!
First, push the `kerblam.toml` file to the remote repository.
Then, fetch its static URL (on github, press the "raw" button on the top right).
It should look something like this: `https://raw.githubusercontent.com/<your username>/<your repo>/refs/head/main/kerblam.toml`

Then, just replace the `{YOUR URL HERE}` below (remove the brackets, `{` and `}`, too!) with the URL of your `kerblam.toml`.

```markdown
![Kerblam!](https://img.shields.io/badge/dynamic/toml?url={YOUR URL HERE}&query=%24.meta.version&prefix=v.&logo=Rocket&logoColor=red&label=Kerblam!)
```

You'll get something like this:
![Kerblam!](https://img.shields.io/badge/dynamic/toml?url=https://raw.githubusercontent.com/TCP-Lab/transportome_profiler/refs/heads/main/kerblam.toml&query=%24.meta.version&prefix=v.&logo=Rocket&logoColor=red&label=Kerblam!)

This dynamically gets the `meta -> version` of your kerblam.toml, so it follows any updates that you make make to it.
