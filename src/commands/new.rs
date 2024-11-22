use std::path::PathBuf;

use anyhow::Result;

use crate::utils::{self, fetch_gitignore, normalize_path, GitCloneMethod, YesNo};
use crate::VERSION;

pub fn create_kerblam_project(dir: &PathBuf) -> Result<()> {
    let dirs_to_create: Vec<&str> = vec![
        "",
        "./.kerblam",
        "./data/in",
        "./data/out",
        "./src/workflows",
        "./src/dockerfiles",
    ];

    let base_data_gitignore: &str =
        "# Ignore everything in this directory\n*\n# Except this file\n!.gitignore\n";

    let mut files_to_create: Vec<(&str, String)> = vec![
        (
            "./kerblam.toml",
            format!("[meta]\nversion = \"{}\"\n", VERSION),
        ),
        ("./data/in/.gitignore", base_data_gitignore.to_string()),
        ("./data/out/.gitignore", base_data_gitignore.to_string()),
        (
            "./data/.gitignore",
            format!("{}\n!in/\n!out/", base_data_gitignore),
        ),
    ];
    // Having this to be a Vec<String> makes all sorts of problems since most
    // commands are hardcoded &str, and we need to go back and forth.
    // Probably can be fixed by generics?
    let mut commands_to_run: Vec<(&str, Vec<String>)> = vec![];
    commands_to_run.push(("git", vec![String::from("init")]));
    let mut gitignore_content: Vec<String> = vec![];
    // We always ignore the .kerblam directory
    gitignore_content.push(".kerblam".to_string());

    // Ask for user input
    // I defined `dirs_to_create` before so that if we ever have to add to them
    // dynamically we can do so here.
    if utils::ask_for::<YesNo>("Do you need Python?").into() {
        // I was once using Vec<(&str, Vec<&str>)> for `commands_to_run`, but
        // PathBuf can become a `String`, and when you `.as_str()`, the original
        // String is freed at the end of this scope, thus rendering the resulting
        // &str reference invalid! The borrow checker complains, so the easiest
        // way that I could think of was to just change the signature to
        // Vec<(&str, Vec<String>)>.
        //
        // Something something generic could probably fix this much more cleanly
        // Or maybe a box?
        commands_to_run.push((
            "python",
            vec!["-m", "venv", "env"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        ));
        gitignore_content
            .push(fetch_gitignore("Python").expect("Failed to fetch Python's gitignore."));
    };

    if utils::ask_for::<YesNo>("Do you need R?").into() {
        gitignore_content.push(fetch_gitignore("R").expect("Failed to fetch R's gitignore"));
    }

    if utils::ask_for::<YesNo>("Do you want to use pre-commit?").into() {
        files_to_create.push(("./pre-commit-config.yaml", String::from("")));
        commands_to_run.push((
            "pre-commit",
            vec![
                "install",
                "--hook-type",
                "pre-commit",
                "--hook-type",
                "commit-msg",
            ]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
        ));
    }

    if utils::ask_for::<YesNo>("Do you want to setup the remote origin of the project?").into() {
        let username = utils::ask("Enter your username: ")?;
        let repo_name: String = dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .trim_matches('"')
            .to_owned();
        let origin_url =
            match utils::ask_for::<GitCloneMethod>("What cloning method would you like?") {
                GitCloneMethod::Ssh => format!("git@github.com:{}/{}.git", username, repo_name),
                GitCloneMethod::Https => {
                    format!("https://github.com/{}/{}.git", username, repo_name)
                }
            };
        commands_to_run.push((
            "git",
            vec!["remote", "add", "origin", origin_url.as_str()]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        ))
    };

    // Write directories
    let dirs_to_create: Vec<PathBuf> = dirs_to_create.into_iter().map(|x| dir.join(x)).collect();

    let results: Vec<Result<String, anyhow::Error>> = dirs_to_create
        .iter()
        .map(|x| normalize_path(x))
        .map(utils::kerblam_create_dir)
        .collect();
    let mut stop = false;
    for res in results {
        match res {
            Ok(msg) => println!("{}", msg),
            Err(msg) => {
                println!("{}", msg);
                stop = true;
            }
        }
    }

    // Write files
    for (file, content) in files_to_create {
        match utils::kerblam_create_file(
            &normalize_path(dir.join(file).as_path()),
            content.as_str(),
            true,
        ) {
            Ok(msg) => println!("{}", msg),
            Err(msg) => {
                println!("{}", msg);
                stop = true
            }
        }
    }

    // Add to gitignore
    match utils::kerblam_create_file(
        normalize_path(dir.join("./.gitignore").as_path()),
        gitignore_content.join("\n").as_str(),
        true,
    ) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => {
            println!("{}", msg);
            stop = true;
        }
    }

    if stop {
        return Ok(());
    }
    // Run commands
    for (command, args) in commands_to_run {
        match utils::run_command(Some(dir), command, args.iter().map(|x| &**x).collect()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "❌ Couldn't execute command '{}': {} {}. Ignoring.",
                    e,
                    command,
                    args.join(" "),
                )
            }
        }
    }

    Ok(())
}
