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
Just copy the following code and add it to the README:
```markdown
![Kerblam!](https://img.shields.io/badge/Kerblam!-v0.5.1-blue?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC0AAAAtCAMAAAANxBKoAAABlVBMVEUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADW1tYNDHwcnNLKFBQgIB/ExMS1tbWMjIufDQ3S0tLOzs6srKyioqJRUVFSS0o0MjIBARqPj48MC3pqaWkIB2MtLS1ybm3U1NS6uroXirqpqamYmJiSkpIPZ4yHh4eFhIV8fHwLWnuBe3kMC3cLCnIHBlwGBlgFBU8EBEVPRkICAi4ADRa+EhIAAAwJCQmJiYnQ0NDKysoZkMK2trYWhLOjo6MTeKMTd6KgoKCbm5uKiIaAgIAPDHhubm4JT20KCW0KCWoIS2cHBUxBQUEEAz9IQT4DAz0DKTpFPTgCAjcCASoBASAXFxcgGRa5ERG1ERGzEBCpDw+hDg4fFA2WDAyLCgouAQFaWloFO1MBHStWBATnwMkoAAAAK3RSTlMA7zRmHcOuDQYK52IwJtWZiXJWQgXw39q2jYBgE/j2187JubKjoJNLSvmSt94WZwAAAvlJREFUSMeF1GdXGkEUgOGliIgIorFH0+u7JBIChEgJamyJvWt6783eS8rvzszAusACvp88x4d7hsvsaqdU57h8oQnobGmtb6xMzwbOkV9jJdvWBRwf7e9uLyzs7B3+o7487miC+AjcvZ3rkNZyttolbKxPv2fyPVrKYKcPhp7oIpPv0FkGN5N5rmd7afAFKH0MH99DihrTK2j3RTICF/Pt0trPUr9AxXyXpkJ3xu6o97tgQJDQm+Xlt6E8vs+FfNrg6kQ1pOuREVSPoydf9YjLpg14gMW1X0IInGZ+9PWr0Xl+R43pxzgM3NgCiekvqfE50hFdT7Ly8Jbo2R/xWYNTl8Ptwk6lgsHUD+Ji2NMlBFZ8ntzZRziXW5kLZsaDom/0yH/G+CSkapS3CvfFCWTxJZgMyqbYVLtLMmzoVywrHaPrrNJX4IHCDyCmF+nXhHXRkzhtCncY+PMig3pu0FfzJG900RBNarTTxrTCEwne69miGV5k8cPst3wOHSfrmJmcCH6Y42NEzzXIX8EFXmFE/q4ZXJrKW4VsY13uzqivF74OD39CbT/0HV/1yQW9Xn8e1O0w+WAG0VJS4P4Mzc7CK+2B7jt6XtFYMhl7Kv4YWMKnsJkXZiW3NgQXxTEKamM2fL8EjzwGv1srykZveBULj6bBZX2Bwbs03cXTQ3HAb9FOGNsS4wt5fw9zv0q9oZo54Gf4UQ95PLbJj/E1HFZ9DRgTuMecPgjfUqlF7Jo1B9wX+JFxmMh7mAoGv9B1pkg2tDoVl7i3G8mjH1mUN3PaspJaqM1NH/sJq2L6QJzEZ4FTCRosuKomdxjYSofDs8DcRPZh8hQd5IbE3qt1ih+MveuVeP2DxOMJAlphgSs1mt3GVWO6yMNGUDZDi1uzJLDNqxbZDLab3mqQB5mExtLYrtU45L10qlfMeSbVQ91eFlfRmnclZyR2VcB5y7pOYhouuSvg2rxHCZG/HHZnsVkVtg7NmkdirS6LzbztTq1EPo9dXRWxqtP7D+wL5neoEOq/AAAAAElFTkSuQmCC&link=https%3A%2F%2Fgithub.com%2FMrHedmad%2Fkerblam)
```

The above link is very long - this is because the Kerblam! logo is baked in as a `base64` image.
You can update the badge's version by directly editing the link (e.g. change
`v0.5.1` to `v0.4.0`) manually.
