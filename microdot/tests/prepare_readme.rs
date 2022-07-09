use askama::Template;
use libmicrodot::util::{compile_input_string_content, git_root};
use std::fs;
use std::path::PathBuf;

#[derive(Template)]
#[template(path = "README.md")]
struct ReadMe {
    fellowship_content: String,
    business_content: String,
    example_content: String,
}

#[test]
fn prepare_readme() {
    // generates the readme for the repo, ensuring that everything works correctly.

    let fellowship_content =
        compile_input_string_content(git_root().unwrap().join("examples/fellowship.txt"));
    let business_content =
        compile_input_string_content(git_root().unwrap().join("examples/business_example_1.txt"));
    let example_content =
        compile_input_string_content(git_root().unwrap().join("examples/readme_example_1.txt"));

    fn content(path: PathBuf) -> String {
        fs::read_to_string(path).expect("couldn't load log file")
    }

    let readme = ReadMe {
        fellowship_content: content(fellowship_content),
        business_content: content(business_content),
        example_content: content(example_content),
    };

    let git_root = git_root().unwrap();

    let readme_path = git_root.join("README.md");
    let readme_content = readme.render().unwrap();
    fs::write(readme_path, readme_content).unwrap();
}
