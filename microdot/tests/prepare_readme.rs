use askama::Template;
use std::fs;
use std::path::Path;
use unfold::Unfold;

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

    let readme = ReadMe {
        fellowship_content: include_str!("../../examples/fellowship.log").to_string(),
        business_content: include_str!("../../examples/business_example_1.log").to_string(),
        example_content: include_str!("../../examples/readme_example_1.log").to_string(),
    };

    let mut current_dir: &Path = &std::env::current_dir().unwrap();
    let mut ancestors = Unfold::new(|path| &path.parent().unwrap(), current_dir);
    let git_root = ancestors
        .find(|&path| {
            // should have a '.git' folder in it.
            fs::read_dir(path)
                .unwrap()
                .flatten()
                .any(|p| p.file_name() == ".git")
        })
        .unwrap();

    let readme_path = git_root.join("README.md");
    let readme_content = readme.render().unwrap();
    fs::write(readme_path, readme_content).unwrap();
}
