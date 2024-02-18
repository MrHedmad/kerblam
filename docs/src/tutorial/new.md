# Creating new projects - `kerblam new`

You can quickly create new kerblam! projects by using `kerblam new`.

Go in a directory where you want to store the new project and run `kerblam new test-project`.
Kerblam! asks you some setup questions:
- If you want to use [Python](https://www.python.org/);
- If you want to use [R](https://www.r-project.org/);
- If you want to use [pre-commit](https://pre-commit.com/);
- If you have a Github account, and would like to setup the `origin` of your
  repository to [github.com](https://github.com).

Say 'yes' to all of these questions to follow along. Kerblam! will then:
- Create the project directory,
- Make a new git repository,
- create the `kerblam.toml` file,
- create all the default project directories,
- make an empty `.pre-commit-config` file for you,
- create a `venv` environment, as well as the `requirements.txt` and `requirements-dev.txt`
  files (if you opted to use Python),
- and setup the `.gitignore` file with appropriate ignores.

> Kerblam! will **NOT** do an `Initial commit` for you!
> You still need to do that manually once you've finished setting up.

You can now start working in your new project, simply `cd test-project`.

Akin to `git`, Kerblam! will look in parent directories for a `kerblam.toml`
file and run there if you call it from a project sub-folder.
Efficient!
