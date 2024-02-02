use std::{
    fs::{copy, read, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::utils::fetch_gitignore;
use anyhow::{Context, Result};

fn is_whitespace(item: &str) -> bool {
    item.chars().all(|x| (x == ' ') | (x == '\t') | (x == '\n'))
}

fn compress_whitespace(items: Vec<&str>) -> Vec<&str> {
    if items.len() < 2 {
        return items;
    }
    // This is so bad but seems to be working??
    // What's wrong with me?
    let mut compressed: Vec<&str> = Vec::with_capacity(items.len());
    let mut previous = items
        .iter()
        .find(|x| !is_whitespace(x))
        .unwrap_or(&items[0]);
    let mut iter = items.iter().skip_while(|x| is_whitespace(x));
    iter.next();
    for item in iter {
        if is_whitespace(previous) & is_whitespace(item) {
            // The two items are both whitespaces.
            // Keep the previous one and continue on
            continue;
        } else {
            // They are not both whitespace.
            compressed.push(previous.trim_end());
            previous = item;
        }
    }

    if !is_whitespace(items[items.len() - 1]) {
        compressed.push(items[items.len() - 1])
    }

    compressed
}

#[test]
fn test_compression() {
    let dirty: Vec<&str> = vec![
        "",
        "",
        "hello!   ",
        "   ",
        "  \n  ",
        "how are you?",
        "\n",
        "fine    thanks",
    ];
    let clean: Vec<&str> = vec!["hello!", "", "how are you?", "", "fine    thanks"];

    assert_eq!(compress_whitespace(dirty), clean);
}

#[test]
fn test_whitespace() {
    assert!(is_whitespace("   "));
    assert!(!is_whitespace("  something  "));
    assert!(is_whitespace("\n\n"));
    assert!(is_whitespace("\t"));
    assert!(is_whitespace("\t\n"));
}

pub fn ignore(target: impl AsRef<Path>, pattern: &str, compress: bool) -> Result<()> {
    let target: &Path = target.as_ref();
    log::debug!("Ignoring {pattern} in file {target:?}");
    if !target.exists() {
        println!("✨ Creating new .gitignore!");
        File::create(target)?;
    };

    // Try to see if the pattern is a path
    let try_path = PathBuf::from(pattern);
    let new_content = if try_path.exists() {
        log::debug!("Adding as path");
        // This is a path. Add it in, but globbed
        let blob = pattern.trim();
        let blob = blob.strip_suffix('/').unwrap_or(blob);
        let blob = blob.strip_prefix("./").unwrap_or(blob);

        blob.to_string() + "\n"
    } else {
        log::debug!("Adding as remote gitignore");
        // This is a language (maybe)
        fetch_gitignore(pattern).context("The specified pattern is not a valid github gitignore")?
    };

    let original = String::from_utf8(read(target)?)?;
    log::debug!("Original gitignore: {:?} bytes", original.len());
    let new = [original, "\n".to_string(), new_content].concat();
    log::debug!("New gitignore: {:?} bytes", new.len());

    let new = if compress {
        log::debug!("Compressing gitignore...");
        // Remove all duplicates except for empty lines and comments
        let mut checks: Vec<&str> = vec![];
        let splits: Vec<&str> = new.split('\n').collect();
        let mut filtered: Vec<&str> = Vec::with_capacity(splits.len());
        for line in splits {
            if line.is_empty() {
                filtered.push(line);
                continue;
            };

            // there is a 'contains' method but it does not work for
            // vectors of strings
            let is_duplicated = checks.iter().any(|x| *x == line);
            if is_duplicated {
                continue;
            };

            checks.push(line);
            filtered.push(line);
        }

        let compressed = compress_whitespace(filtered).join("\n");
        log::debug!("Compressed gitignore: {:?} bytes", compressed.len());
        compressed
    } else {
        new
    };

    let temp = tempfile::NamedTempFile::new()?;
    let mut handle = File::create(temp.path())?;
    log::debug!("Writing to temp file {:?}", temp.path());
    handle
        .write_all(new.as_bytes())
        .context("Failed writing to temporary file")?;

    // I copy the file from a temp one so it's an atomic transaction
    // and I don't die while doing it for any reason.
    copy(temp.path(), target)?;

    temp.close()?;

    Ok(())
}
