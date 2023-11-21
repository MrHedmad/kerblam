use std::error::Error;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;

/// Create a directory, and prepare an output message.
///
/// Creates a dir only if it does not exist.
///
/// Output messages are wrapped in `Ok` or `Err` in order to differentiate
/// between the two. On Err, the program is expected to exit, since the
/// folder does not exist AND it cannot be created.
pub fn kerblam_create_dir(dir: &PathBuf) -> Result<String, String> {
    if dir.exists() && dir.is_dir() {
        return Ok(format!("🔷 {:?} was already there!", dir));
    }
    if dir.exists() && dir.is_file() {
        return Err(format!("❌ {:?} is a file!", dir));
    }

    match fs::create_dir_all(dir) {
        Ok(_) => Ok(format!("✅ {:?} created!", dir)),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(format!("❌ No permission to write {:?}!", dir)),
            kind => Err(format!("❌ Failed to write {:?} - {:?}", dir, kind)),
        },
    }
}

pub fn kerblam_create_file(
    file: &PathBuf,
    content: &str,
    overwrite: bool,
) -> Result<String, String> {
    if file.exists() && !overwrite {
        return Err(format!("❌ {:?} was already there!", file));
    }

    if file.exists() && file.is_dir() {
        return Err(format!("❌ {:?} is a directory!", file));
    }

    match fs::write(file, content) {
        Ok(_) => Ok(format!("✅ {:?} created!", file)),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(format!("❌ No permission to write {:?}!", file)),
            kind => Err(format!("❌ Failed to write {:?} - {:?}", file, kind)),
        },
    }
}

pub fn ask(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    Ok(buffer.trim().to_owned())
}

pub enum UserResponse {
    Yes,
    No,
    Abort,
    Explain,
}

impl UserResponse {
    pub fn parse(other: String) -> Option<UserResponse> {
        match other.to_lowercase().chars().nth(0) {
            None => None,
            Some(char) => match char {
                'y' => Some(Self::Yes),
                'n' => Some(Self::No),
                'a' => Some(Self::Abort),
                'e' => Some(Self::Explain),
                _ => None,
            },
        }
    }

    pub fn ask_options(prompt: &str, explaination: &str) -> UserResponse {
        let prompt = format!("{} [yes/no/abort/explain]: ", prompt);
        let mut result: Option<UserResponse>;
        loop {
            match ask(&prompt) {
                Err(_) => {
                    println!("Error reading input. Try again?");
                    continue;
                }
                Ok(string) => result = Self::parse(string),
            };

            match result {
                None => {
                    println!("Invalid input. Please enter y/n/a/e.");
                    continue;
                }
                Some(Self::Abort) => std::process::abort(),
                Some(Self::Explain) => println!("{}", explaination),
                _ => break,
            }
        }

        result.unwrap()
    }
}

impl Into<char> for UserResponse {
    fn into(self) -> char {
        match self {
            Self::Yes => 'y',
            Self::No => 'n',
            Self::Abort => 'a',
            Self::Explain => 'e',
        }
    }
}

impl Into<bool> for UserResponse {
    fn into(self) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
            Self::Abort => false,
            Self::Explain => false,
        }
    }
}
