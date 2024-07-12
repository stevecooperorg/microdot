use crate::graphviz::{compile, DisplayMode, GraphVizExporter};
use crate::repl::repl;
use crate::Interaction;
use anyhow::{anyhow, Context, Result};
use microdot_core::graph::Graph;
use std::collections::VecDeque;
use std::path::*;
use std::sync::{Arc, RwLock};
use unfold::Unfold;

pub fn git_root() -> Result<PathBuf> {
    let current_dir: &Path = &std::env::current_dir()?;
    let mut ancestors = Unfold::new(|path| path.parent().unwrap(), current_dir);

    fn has_git_dir(path: &Path) -> bool {
        std::fs::read_dir(path)
            .unwrap()
            .flatten()
            .any(|p| p.file_name() == ".git")
    }

    let git_root = ancestors
        .find(|&path| has_git_dir(path))
        .ok_or_else(|| anyhow!("could not find the git root"))?;

    Ok(git_root.into())
}

pub fn compile_input_string_content(text_file: PathBuf) -> PathBuf {
    assert!(
        text_file.exists(),
        "text file does not exist: '{}'",
        text_file.to_string_lossy()
    );

    // read the file as lines and run it through the repl;
    let graph = Arc::new(RwLock::new(Graph::new()));

    let text_content = std::fs::read_to_string(&text_file).expect("could not read file");
    let lines: VecDeque<_> = text_content.lines().map(|l| l.to_string()).collect();
    let mut auto_interaction = AutoInteraction::new(lines);
    let tmp_json = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(text_file.file_name().unwrap());

    repl(&mut auto_interaction, &tmp_json, graph.clone()).expect("error in repl");

    let temp_json = std::fs::read_to_string(&tmp_json).expect("could not read json file");
    let final_json_path = text_file.with_extension("json");
    write_if_different(final_json_path, temp_json).expect("could not write json file");

    let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
    let graph = graph.read().unwrap();
    let exported = exporter.export_dot(&graph);

    let dot_file = text_file.with_extension("dot");
    write_if_different(&dot_file, exported).expect("could not write dot file");

    let log_file = text_file.with_extension("log");
    write_if_different(&log_file, auto_interaction.log()).expect("could not write log file");

    compile(&dot_file)
        .unwrap_or_else(|e| panic!("Could not compile '{}': {}", dot_file.to_string_lossy(), e));

    log_file
}

pub fn write_if_different<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();

    let needs_write = match std::fs::read(path) {
        Ok(current_content) => current_content != contents,
        Err(_) => true,
    };

    if needs_write {
        std::fs::write(path, contents)
            .with_context(|| format!("could not write to file '{}'", path.to_string_lossy()))
    } else {
        Ok(())
    }
}

struct AutoInteraction {
    lines: VecDeque<String>,
    log: String,
}

impl AutoInteraction {
    fn new(lines: VecDeque<String>) -> Self {
        Self {
            lines,
            log: Default::default(),
        }
    }

    fn log(&self) -> String {
        self.log.clone()
    }
}

impl Interaction for AutoInteraction {
    fn read(&mut self, _prompt: &str) -> rustyline::Result<String> {
        match self.lines.pop_front() {
            Some(line) => Ok(line),
            None => Err(rustyline::error::ReadlineError::Eof),
        }
    }

    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool {
        self.log.push_str(">> ");
        self.log.push_str(&history.into());
        self.log.push('\n');
        true
    }

    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S) {
        self.log.push_str(&message.into());
        self.log.push('\n');
    }

    fn should_compile(&self) -> bool {
        false
    }
}

#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn write_if_different_should_write_if_file_does_not_exist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let contents = "hello world";

        write_if_different(&path, contents).unwrap();

        check_content_equal(&path, contents);
    }

    #[test]
    fn write_if_different_should_not_write_if_file_exists_and_contents_are_the_same() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let contents = "hello world";

        std::fs::write(&path, contents).unwrap();
        let original_time = get_file_modified_time(&path).unwrap();

        write_if_different(&path, contents).unwrap();
        check_content_equal(&path, contents);
        let new_time = get_file_modified_time(&path).unwrap();
        assert_eq!(original_time, new_time);
    }

    #[test]
    fn write_if_different_should_write_if_file_exists_and_contents_are_different() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        std::fs::write(&path, "old content").unwrap();
        let original_time = get_file_modified_time(&path).unwrap();

        write_if_different(&path, "new content").unwrap();
        check_content_equal(&path, "new content");
        let new_time = get_file_modified_time(&path).unwrap();
        assert_ne!(original_time, new_time);
    }

    fn get_file_modified_time(path: &PathBuf) -> Option<std::time::SystemTime> {
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        Some(modified)
    }

    fn check_content_equal(path: &PathBuf, contents: &str) {
        let mut file = File::open(path).unwrap();
        let mut read_contents = String::new();
        file.read_to_string(&mut read_contents).unwrap();

        assert_eq!(contents, read_contents);
    }
}
