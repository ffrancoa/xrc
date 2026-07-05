use std::fs;

use assert_cmd::Command;

fn xrc() -> Command {
    Command::cargo_bin("xrc").unwrap()
}

#[test]
fn help_lists_both_commands() {
    let assert = xrc().arg("--help").assert().success();
    let output = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    // clap strips the doc comment's trailing period in help output
    assert!(output.contains("Scrape coding sites and generate Rust problem templates"));
    assert!(output.contains("pull"));
    assert!(output.contains("check"));
}

#[test]
fn short_help_works() {
    xrc().arg("-h").assert().success();
}

#[test]
fn pull_rejects_unsupported_site() {
    xrc()
        .args(["pull", "https://example.com/problem/1"])
        .assert()
        .failure()
        .code(1)
        .stdout("error: unsupported site 'example.com'\n");
}

#[test]
fn check_fails_without_exercises_dir() {
    let tmp = tempdir("no_exercises");
    xrc()
        .current_dir(&tmp)
        .arg("check")
        .assert()
        .failure()
        .code(1)
        .stdout("error: exercises directory 'exercises' not found\n");
}

#[test]
fn check_reports_empty_exercises_dir() {
    let tmp = tempdir("empty_exercises");
    fs::create_dir_all(tmp.join("exercises")).unwrap();
    xrc()
        .current_dir(&tmp)
        .arg("check")
        .assert()
        .success()
        .stdout("no rust projects found in exercises directory\n");
}

#[test]
fn check_skips_completed_and_merges_progress() {
    let tmp = tempdir("progress_merge");
    let exercises = tmp.join("exercises");
    fs::create_dir_all(exercises.join("some_project")).unwrap();
    fs::write(exercises.join("some_project/Cargo.toml"), "[package]\n").unwrap();
    // pre-existing progress marks the project done and has an unrelated entry
    fs::write(
        exercises.join(".xrc_progress.json"),
        "{\n  \"other_project\": false,\n  \"some_project\": true\n}",
    )
    .unwrap();

    xrc()
        .current_dir(&tmp)
        .arg("check")
        .assert()
        .success()
        .stdout("summary:\n- `some_project` (done)\n");

    let progress = fs::read_to_string(exercises.join(".xrc_progress.json")).unwrap();
    assert_eq!(
        progress,
        "{\n  \"other_project\": false,\n  \"some_project\": true\n}"
    );
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("xrc_test_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
