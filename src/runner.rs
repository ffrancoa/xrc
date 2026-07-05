use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use anstream::println;
use regex::Regex;

use crate::output::{YELLOW, paint};

/// Handles running and parsing cargo tests.
pub struct TestRunner<'a> {
    project_dir: &'a Path,
    verbose: bool,
}

struct CommandResult {
    success: bool,
    output: String,
}

/// Extract test names from `cargo test -- --list` output.
fn parse_test_list(output: &str) -> Vec<String> {
    // match patterns like "test name ... ok" or "name: test"
    static PATTERN_1: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:test\s+)?(?P<name>[\w:]+)\s*(?:\.\.\.|:)\s*(?:ok|FAILED|ignored)?$")
            .unwrap()
    });
    static PATTERN_2: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?P<name>[\w:]+)\s*:\s*test$").unwrap());

    let mut test_names: Vec<String> = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        for pattern in [&*PATTERN_1, &*PATTERN_2] {
            if let Some(caps) = pattern.captures(line) {
                let name = &caps["name"];
                if !name.starts_with("running") && !test_names.iter().any(|n| n == name) {
                    test_names.push(name.to_string());
                }
                break;
            }
        }
    }
    test_names
}

impl<'a> TestRunner<'a> {
    pub fn new(project_dir: &'a Path, verbose: bool) -> Self {
        Self {
            project_dir,
            verbose,
        }
    }

    /// Run cargo tests and return (success, test_results).
    pub fn run_tests(&self) -> (bool, Vec<(String, bool)>) {
        let test_names = self.enumerate_tests();
        if test_names.is_empty() {
            return self.fallback_test_run();
        }
        self.run_individual_tests(&test_names)
    }

    fn run_command(&self, args: &[&str]) -> CommandResult {
        // stdout and stderr are concatenated, mirroring Python's
        // stderr=subprocess.STDOUT (test names appear on stdout either way)
        match Command::new("cargo")
            .args(args)
            .current_dir(self.project_dir)
            .output()
        {
            Ok(out) => {
                let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&out.stderr));
                CommandResult {
                    success: out.status.success(),
                    output: text,
                }
            }
            Err(e) => CommandResult {
                success: false,
                output: format!("error running cargo: {e}"),
            },
        }
    }

    fn enumerate_tests(&self) -> Vec<String> {
        let result = self.run_command(&["test", "--", "--list"]);
        parse_test_list(&result.output)
    }

    /// Fallback to a single cargo test run when enumeration fails.
    fn fallback_test_run(&self) -> (bool, Vec<(String, bool)>) {
        if self.verbose {
            println!(
                "{}",
                paint(
                    YELLOW,
                    "warning: no tests enumerated. running single cargo test."
                )
            );
        }

        let result = self.run_command(&["test", "-q"]);
        if !result.success && self.verbose {
            println!("{}", result.output);
        }
        (result.success, Vec::new())
    }

    /// Run each test individually and collect results.
    fn run_individual_tests(&self, test_names: &[String]) -> (bool, Vec<(String, bool)>) {
        let mut tests: Vec<(String, bool)> = Vec::new();
        let mut all_passed = true;

        for (i, name) in test_names.iter().enumerate() {
            if self.verbose {
                println!("running test {}/{}: {}", i + 1, test_names.len(), name);
            }

            let result = self.run_command(&["test", name, "--", "--exact", "--nocapture"]);
            tests.push((name.clone(), result.success));

            if !result.success {
                all_passed = false;
                if self.verbose {
                    println!("{}", result.output);
                }
            } else if self.verbose {
                println!("  -> passed");
            }
        }

        (all_passed, tests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_list_format() {
        let output = "\
running 2 tests
tests::example_1: test
tests::example_2: test

2 tests, 0 benchmarks
";
        assert_eq!(
            parse_test_list(output),
            vec!["tests::example_1", "tests::example_2"]
        );
    }

    #[test]
    fn parses_run_format_and_dedupes() {
        let output = "\
test tests::example ... ok
test tests::example ... ok
test other ... FAILED
";
        assert_eq!(parse_test_list(output), vec!["tests::example", "other"]);
    }

    #[test]
    fn ignores_noise_lines() {
        let output = "\
   Compiling foo v0.1.0 (/tmp/foo)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.5s
     Running unittests src/lib.rs

running 1 test
";
        assert!(parse_test_list(output).is_empty());
    }
}
