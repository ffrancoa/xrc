use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anstream::println;

use crate::output::{BOLD_UNDERLINE, CYAN, GREEN, RED, YELLOW, paint};
use crate::progress::ProgressTracker;
use crate::runner::TestRunner;
use crate::{renderer, scrapers, utils, writer};

pub fn cmd_pull(url: &str) -> i32 {
    let Some((source, scraper_fn)) = scrapers::find_scraper(url) else {
        let netloc = scrapers::netloc(url);
        println!(
            "{}",
            paint(RED, &format!("error: unsupported site '{netloc}'"))
        );
        return 1;
    };

    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => {
            println!("{}", paint(RED, &format!("error creating project: {e}")));
            return 1;
        }
    };
    let exercises_dir = if cwd.file_name() == Some(OsStr::new("exercises")) {
        cwd
    } else {
        cwd.join("exercises")
    };

    let create = || -> anyhow::Result<String> {
        let data = scraper_fn(url)?;
        let files = renderer::render_rust_template(&data, source)?;
        let project_name = utils::to_snake_case(&data.title);
        writer::write_rust_project(&project_name, &files, &exercises_dir)?;
        Ok(project_name)
    };

    match create() {
        Ok(project_name) => {
            println!(
                "{}",
                paint(
                    GREEN,
                    &format!("successfully created project: {project_name}")
                )
            );
            0
        }
        Err(e) => {
            println!("{}", paint(RED, &format!("error creating project: {e}")));
            1
        }
    }
}

/// Find all Cargo.toml files under `dir`, recursively (like Path.rglob).
fn find_cargo_tomls(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            found.extend(find_cargo_tomls(&path));
        } else if path.file_name() == Some(OsStr::new("Cargo.toml")) {
            found.push(path);
        }
    }
    found
}

fn check_project(project_path: &Path, verbose: bool) -> (String, bool) {
    let project_dir = project_path.parent().unwrap_or(Path::new("."));
    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    println!("testing `{project_name}`...");

    let runner = TestRunner::new(project_dir, verbose);
    let (success, tests) = runner.run_tests();

    if tests.is_empty() {
        println!("- no individual tests detected");
    } else {
        for (idx, (name, passed)) in tests.iter().enumerate() {
            let (style, status) = if *passed {
                (GREEN, "passed")
            } else {
                (RED, "failed")
            };
            println!(
                "- {}:{}",
                paint(CYAN, &format!("test {}", idx + 1)),
                paint(style, &format!(" {status} ({name}) "))
            );
        }
    }
    println!();

    (project_name, success)
}

fn print_summary(results: &[(String, bool)]) {
    println!("{}", paint(BOLD_UNDERLINE, "summary:"));
    for (name, success) in results {
        let (style, status_text) = if *success {
            (GREEN, "(done)")
        } else {
            (YELLOW, "(pending)")
        };
        println!("- `{name}` {}", paint(style, status_text));
    }
}

pub fn cmd_check(recheck: bool, verbose: bool) -> i32 {
    let exercises_dir = Path::new("exercises");
    if !exercises_dir.exists() {
        println!(
            "{}",
            paint(RED, "error: exercises directory 'exercises' not found")
        );
        return 1;
    }

    let projects = find_cargo_tomls(exercises_dir);
    if projects.is_empty() {
        println!(
            "{}",
            paint(YELLOW, "no rust projects found in exercises directory")
        );
        return 0;
    }

    let mut tracker = ProgressTracker::new(exercises_dir.join(".xrc_progress.json"));
    let mut results: Vec<(String, bool)> = Vec::new();
    let upsert = |results: &mut Vec<(String, bool)>, name: String, success: bool| {
        if let Some(entry) = results.iter_mut().find(|(n, _)| *n == name) {
            entry.1 = success;
        } else {
            results.push((name, success));
        }
    };

    for project_path in &projects {
        let project_name = project_path
            .parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        // skip if already completed and not rechecking
        if !recheck && tracker.is_completed(&project_name) {
            upsert(&mut results, project_name, true);
            continue;
        }

        let (project_name, success) = check_project(project_path, verbose);
        upsert(&mut results, project_name, success);
    }

    tracker.save_progress(&results);
    print_summary(&results);

    if results.iter().any(|(_, success)| !success) {
        return 1;
    }
    0
}
