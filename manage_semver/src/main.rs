use clap::Parser;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "semver.txt")]
    semver_file_path: PathBuf,
}

fn main() {
    eprintln!(
        "{} - increment the patch version stored in a semver tag file",
        env!("CARGO_BIN_NAME")
    );

    let Args { semver_file_path } = Args::parse();

    eprintln!("semver_file_path: {:?}", semver_file_path);

    // Read the file
    let semver_file_contents = std::fs::read_to_string(&semver_file_path)
        .unwrap_or_else(|_| panic!("Failed to read file: {:?}", semver_file_path))
        .trim()
        .to_string();

    let new_semver = increment_in_content(&semver_file_contents, "CURRENT_DOCKER_SEMVER_TAG");

    eprintln!("new_semver: {:?}", new_semver);

    // write to stdout so it can be piped
    print!("{}", new_semver);
}

fn increment_in_content(semver_file_contents: &str, semver_key: &str) -> String {
    eprintln!("semver_file_contents: {:?}", semver_file_contents);

    let mut variables = BTreeMap::new();

    // Parse the file contents
    for line in semver_file_contents.lines() {
        let line = line.trim();
        let parts = line.split('=').collect::<Vec<&str>>();
        if parts.len() != 2 {
            panic!("Invalid line: {:?}", line);
        }
        let key = parts[0].trim();
        let value = parts[1].trim();
        eprintln!("key: {:?}, value: {:?}", key, value);
        variables.insert(key, value);
    }

    let semver = match variables.get(semver_key) {
        Some(semver) => semver,
        None => {
            panic!("SEMVER not found in file");
        }
    };

    let semver = semver.split('.').collect::<Vec<&str>>();

    // should be three integers;
    // major, minor, patch
    if semver.len() != 3 {
        panic!("Invalid semver file contents: {:?}", semver_file_contents);
    }

    let major = semver[0].parse::<u32>().unwrap();
    let minor = semver[1].parse::<u32>().unwrap();
    let patch = semver[2].parse::<u32>().unwrap();

    eprintln!("major: {}, minor: {}, patch: {}", major, minor, patch);

    // Increment the patch version
    let new_patch = patch + 1;

    // Replace the old patch version with the new one
    let new_semver = format!("{}.{}.{}", major, minor, new_patch);
    variables.insert(semver_key, &new_semver);

    let mut new_semver_file_contents = String::new();
    for (key, value) in variables.iter() {
        new_semver_file_contents.push_str(&format!("{}={}\n", key, value));
    }
    new_semver_file_contents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_in_content() {
        let semver_file_contents = r#"X=Y
CURRENT_DOCKER_SEMVER_TAG=1.2.3
A=B
"#;
        let expected = r#"A=B
CURRENT_DOCKER_SEMVER_TAG=1.2.4
X=Y
"#;

        let actual = increment_in_content(semver_file_contents, "CURRENT_DOCKER_SEMVER_TAG");
        assert_eq!(actual, expected);
    }
}
