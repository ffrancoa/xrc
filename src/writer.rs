use std::fs;
use std::path::Path;

use anyhow::Result;

/// Create a Rust project directory with the given files. `files` maps paths
/// (relative to the project root) to their contents. Existing files are
/// silently overwritten.
pub fn write_rust_project(
    project_name: &str,
    files: &[(String, String)],
    base_dir: &Path,
) -> Result<()> {
    let root_dir = base_dir.join(project_name);
    fs::create_dir_all(&root_dir)?;

    for (rel_path, content) in files {
        let file_path = root_dir.join(rel_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, format!("{}\n", content.trim()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_files_with_trailing_newline() {
        let tmp = std::env::temp_dir().join(format!("xrc_writer_test_{}", std::process::id()));
        let files = vec![
            ("src/lib.rs".to_string(), "  fn x() {}  \n\n".to_string()),
            ("Cargo.toml".to_string(), "[package]".to_string()),
        ];
        write_rust_project("proj", &files, &tmp).unwrap();
        assert_eq!(
            fs::read_to_string(tmp.join("proj/src/lib.rs")).unwrap(),
            "fn x() {}\n"
        );
        assert_eq!(
            fs::read_to_string(tmp.join("proj/Cargo.toml")).unwrap(),
            "[package]\n"
        );
        fs::remove_dir_all(&tmp).unwrap();
    }
}
