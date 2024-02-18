use git2;
use paste;
use std::collections::HashMap;
use std::env::set_current_dir;
use std::mem::drop;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};
use tempfile::TempDir;

const EXAMPLE_REPO: &'static str = "https://github.com/MrHedmad/kerblam-examples.git";

/// Setup local tests by cloning the example repo.
fn setup_test() -> TempDir {
    let target = TempDir::new().expect("Could not create temp dir");

    let mut clone_options = git2::FetchOptions::new();
    clone_options.depth(1);
    clone_options.download_tags(git2::AutotagOption::None);

    let mut fetcher = git2::build::RepoBuilder::new();
    fetcher.fetch_options(clone_options);

    fetcher
        .clone(EXAMPLE_REPO, &target.path())
        .expect("Could not clone remote repo.");

    target
}

// This is hard, even for me, but all the examples look the same, so a macro
// that generates the tests makes a lot of sense.
//
// To run an example, you:
// - Clone the repo;
// - Move into the example folder;
// - Execute the `run` shell script;
// - Check if the repo is still the same;
//   - The `run` script should run the example and roll it back.
//
// This is a rough integration test to see if all the various invocations of
// kerblam! run correctly.
// If a test fails, check the single example to see why it failed and fix the
// root cause.

type Content = HashMap<PathBuf, Vec<u8>>;

// Take a snapshot of the directory, returning the current content
fn snapshot(dir: &PathBuf) -> io::Result<Content> {
    let mut content: Content = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            continue;
        }

        content.insert(entry.path().to_path_buf(), fs::read(entry.path())?);
    }

    Ok(content)
}

fn two_way_check<T>(a: &Vec<T>, b: &Vec<T>) -> bool
where
    T: PartialEq,
{
    a.iter().all(|item| b.contains(item)) & b.iter().all(|item| a.contains(item))
}

fn check_identical(old_snap: Content, target: impl AsRef<Path>) -> io::Result<()> {
    let new_snap = snapshot(&target.as_ref().to_owned().to_path_buf()).unwrap();

    let old_files: Vec<PathBuf> = old_snap.keys().cloned().collect();
    let new_files: Vec<PathBuf> = new_snap.keys().cloned().collect();
    if !two_way_check(&old_files, &new_files) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Number of files differ: old: {:?}, new: {:?}",
                old_files, new_files
            ),
        ));
    }

    for path in old_files {
        let current = fs::read(&path)?;

        if current != *old_snap.get(&path).unwrap() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("File {:?} has modified data!", path),
            ));
        }
    }

    Ok(())
}

macro_rules! run_example_named {
    ($name:literal) => {
        paste::item! {

            #[test]
            fn [< example_ $name >] () {
                let repo = setup_test();
                let target = repo.path().join(format!("examples/{}", $name));

                // Move into the example
                println!("{:?}", target);
                set_current_dir(&target).unwrap();

                // Take a snapshot of the contents of the repo
                let snap = snapshot(&target).expect("Could not take snapshot of repo");

                // Run the things
                let code = Command::new("bash")
                    .args([target.join("run")])
                    .output()
                    .expect("Could not collect child output");

                eprintln!(
                    "Command Output\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                    String::from_utf8_lossy(&code.stdout),
                    String::from_utf8_lossy(&code.stderr),
                );

                assert!(code.status.success());

                assert!(check_identical(snap, &target).is_ok());

                // The 'repo' variable gets dropped here, so the tempdir is cleaned up
                // SPECIFICALLY here.
                drop(repo);
            }
        }
    };
}

// TODO: This is repeated from above with only small differences.
// Can we avaoid it?
macro_rules! run_failing_example_named {
    ($name:literal) => {
        paste::item! {

            #[test]
            fn [< failing_example_ $name >] () {
                let repo = setup_test();
                let target = repo.path().join(format!("examples/{}", $name));

                // Move into the example
                println!("{:?}", target);
                set_current_dir(&target).unwrap();

                // Take a snapshot of the contents of the repo
                let snap = snapshot(&target).expect("Could not take snapshot of repo");

                // Run the things
                let code = Command::new("bash")
                    .args([target.join("run")])
                    .output()
                    .expect("Could not collect child output");

                eprintln!(
                    "Command Output\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                    String::from_utf8_lossy(&code.stdout),
                    String::from_utf8_lossy(&code.stderr),
                );

                assert!(! code.status.success());

                assert!(check_identical(snap, &target).is_ok());

                // The 'repo' variable gets dropped here, so the tempdir is cleaned up
                // SPECIFICALLY here.
                drop(repo);
            }
        }
    };
}

run_example_named!("basic_pipes");
run_example_named!("basic_containers");
run_failing_example_named!("safe_failure");
