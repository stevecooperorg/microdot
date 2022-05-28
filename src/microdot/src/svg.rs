use microdot_core::CommandResult;
use std::path::Path;

const GAPPLIN_PATH: &str = "/Applications/Gapplin.app/Contents/MacOS/Gapplin";

pub fn open_in_gapplin(svg_path: &Path) -> CommandResult {
    let viewer = GAPPLIN_PATH;
    let svg_path = &svg_path.to_string_lossy().to_string();
    if Path::new(viewer).exists() {
        let mut cmd = std::process::Command::new(viewer);
        cmd.arg(svg_path);
        match cmd.spawn() {
            Ok(_) => CommandResult::new(format!("Opened {} in {}", svg_path, viewer)),
            Err(e) => CommandResult::new(format!(
                "Could not open {} in {}: {}",
                svg_path,
                viewer,
                e.to_string()
            )),
        }
    } else {
        CommandResult::new(format!("Could not open {} in {}", svg_path, viewer))
    }
}
