# The Kerblam.toml file

The `kerblam.toml` file is the control center of kerblam!
All of its configuration is found there.
Here is what fields are available, and what they do.

> [!IMPORTANT]
> Extra fields not found here are silently ignored.
> This means that you must be careful of typos!

The fields are annotated where possible with the default value.
```toml
[meta] # Metadata regarding kerblam!
version = "0.4.0"
# Kerblam! will check this version and give you a warning
# if you are not running the same executable.
# To save you headaches!

##### --- #####
[data] # Data paths, profiles, etc...
[data.paths]
input: "./data/in"
output: "./data/out"
intermediate: "./data"
temporary: "/tmp" # This field is ignored by kerblam! but still parsed.

[data.profiles] # Specify profiles here
profile_name = {
    "original_name" = "profile_name",
    "other_name" = "other_profile_name"
}

# Or, alternatively
[data.profiles.profile_name]
"original_name" = "profile_name"
"other_name" = "other_profile_name"
# Any number of profiles can be specified.

[data.remote] # Specify how to fetch remote data
"url_to_fetch" = "file_to_save_to"
# there can be any number of "url" = "file" entries here.

##### --- #####
[code] # Where to look for containers and pipes
env_dir = "./src/dockerfiles"
pipes_dir = "./src/pipes"

[execution] # How to execute the pipelines
backend = "docker" # or "podman", the backend to use to build and run containers
workdir = "/" # The working directory inside all built containers
```

Note that this does not want to be a valid TOML, just a reference.
