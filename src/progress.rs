use std::fs;
use std::path::PathBuf;

use anstream::println;
use serde_json::{Map, Value};

use crate::output::{YELLOW, paint};

/// Manages exercise progress tracking. The progress file is a flat JSON
/// object mapping project names to booleans; the schema (and 2-space pretty
/// printing) is kept stable so existing files keep working.
pub struct ProgressTracker {
    progress_file: PathBuf,
    progress: Map<String, Value>,
}

impl ProgressTracker {
    pub fn new(progress_file: PathBuf) -> Self {
        let progress = Self::load_progress(&progress_file);
        Self {
            progress_file,
            progress,
        }
    }

    fn load_progress(path: &PathBuf) -> Map<String, Value> {
        if !path.exists() {
            return Map::new();
        }
        let loaded = fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|text| {
                serde_json::from_str::<Map<String, Value>>(&text).map_err(|e| e.to_string())
            });
        match loaded {
            Ok(map) => map,
            Err(e) => {
                println!(
                    "{}",
                    paint(
                        YELLOW,
                        &format!("warning: could not load progress file: {e}")
                    )
                );
                Map::new()
            }
        }
    }

    pub fn save_progress(&mut self, results: &[(String, bool)]) {
        for (name, success) in results {
            self.progress.insert(name.clone(), Value::Bool(*success));
        }

        let write = || -> std::io::Result<()> {
            if let Some(parent) = self.progress_file.parent() {
                fs::create_dir_all(parent)?;
            }
            let text = serde_json::to_string_pretty(&self.progress)?;
            fs::write(&self.progress_file, text)
        };
        if let Err(e) = write() {
            println!(
                "{}",
                paint(YELLOW, &format!("warning: could not save progress: {e}"))
            );
        }
    }

    pub fn is_completed(&self, project_name: &str) -> bool {
        self.progress
            .get(project_name)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_pretty_json_file() {
        let tmp = std::env::temp_dir().join(format!("xrc_progress_test_{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();
        let file = tmp.join(".xrc_progress.json");
        // the exact 2-space pretty JSON we persist
        fs::write(&file, "{\n  \"two_sum\": true,\n  \"a_plus_b\": false\n}").unwrap();

        let mut tracker = ProgressTracker::new(file.clone());
        assert!(tracker.is_completed("two_sum"));
        assert!(!tracker.is_completed("a_plus_b"));
        assert!(!tracker.is_completed("unknown"));

        tracker.save_progress(&[("a_plus_b".to_string(), true)]);
        let text = fs::read_to_string(&file).unwrap();
        assert_eq!(text, "{\n  \"two_sum\": true,\n  \"a_plus_b\": true\n}");

        fs::remove_dir_all(&tmp).unwrap();
    }
}
