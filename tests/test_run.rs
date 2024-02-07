use crate::utils::{assert_ok, init_log, setup_workdir, File};
use chwd;
use kerblam::kerblam;
use rusty_fork::rusty_fork_test;

// As cargo test runs a new *thread* at each test, multiple calls to kerblam!
// may cause the `ctrlc` signal hook to be set multiple times (as it is
// registered for the whole *process*, not each *thread*).
// rusty-fork makes it so that each test is ran in its own *process*, so
// this problem does not occur.

static TEST_KERBLAM_TOML: &'static str = r#"
[data.remote]
"https://raw.githubusercontent.com/MrHedmad/kerblam/main/README.md" = "input_data.txt"
"https://raw.githubusercontent.com/BurntSushi/ripgrep/master/README.md" = "alternate_input_data.txt"

[data.profiles.alternate]
"input_data.txt" = "alternate_input_data.txt"

"#;

static TEST_SHELL_PIPE: &'static str = r#"
echo "Running shell pipe"
mkdir -p ./data/out/
cat ./data/in/input_data.txt | wc -l > ./data/out/line_count.txt
"#;

static TEST_ERROR_SHELL_PIPE: &'static str = r#"
echo "Running error shell pipe"
exit 1
"#;

static TEST_MAKE_PIPE: &'static str = r#"
.RECIPEPREFIX = > 
all: ./data/out/line_count.txt

./data/out/line_count.txt: ./data/in/input_data.txt
> echo "Running make pipe"
> mkdir -p ${@D}
> cat $< | wc -l > $@

"#;

static TEST_DOCKER_FILE: &'static str = r#"
FROM ubuntu:latest

COPY . .
"#;

fn get_default_files() -> Vec<File> {
    vec![
        File::new("kerblam.toml", TEST_KERBLAM_TOML),
        File::new("src/pipes/make_pipe.makefile", TEST_MAKE_PIPE),
        File::new("src/pipes/shell_pipe.sh", TEST_SHELL_PIPE),
        File::new("src/dockerfiles/default.dockerfile", TEST_DOCKER_FILE),
        File::new("src/pipes/error.sh", TEST_ERROR_SHELL_PIPE),
    ]
}

rusty_fork_test! {
    #[test]
    fn test_shell_analysis() {
        init_log();
        let default_files = get_default_files();
        let temp_dir = setup_workdir(default_files.iter());
        let _flag = chwd::ChangeWorkingDirectory::change(&temp_dir).unwrap();

        assert_ok(kerblam(vec!["", "data", "fetch"].iter()));
        assert_ok(kerblam(vec!["", "run", "shell_pipe", "--local"].iter()));
    }
}

rusty_fork_test! {
    #[test]
    fn test_make_analysis() {
        init_log();
        let default_files = get_default_files();
        let temp_dir = setup_workdir(default_files.iter());
        let _flag = chwd::ChangeWorkingDirectory::change(&temp_dir).unwrap();

        assert_ok(kerblam(vec!["", "data", "fetch"].iter()));
        assert_ok(kerblam(vec!["", "run", "make_pipe", "--local"].iter()));
    }
}

rusty_fork_test! {
    #[test]
    #[should_panic]
    fn test_erroring_pipe() {
        init_log();
        let default_files = get_default_files();
        let temp_dir = setup_workdir(default_files.iter());
        let _flag = chwd::ChangeWorkingDirectory::change(&temp_dir).unwrap();

        assert_ok(kerblam(vec!["", "data", "fetch"].iter()));
        assert_ok(kerblam(vec!["", "run", "error", "--local"].iter()));
    }
}