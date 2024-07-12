use clap::Parser;
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

    eprintln!("semver_file_contents: {:?}", semver_file_contents);

    // Parse the file contents
    let semver = semver_file_contents.split('.').collect::<Vec<&str>>();

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

    // generate the new string
    let new_semver = format!("{}.{}.{}", major, minor, new_patch);

    eprintln!("new_semver: {:?}", new_semver);

    // write to stdout so it can be piped
    print!("{}", new_semver);
}
