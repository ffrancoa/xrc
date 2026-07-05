use anyhow::{Result, bail};
use minijinja::{Environment, context};
use serde::Serialize;

use crate::scrapers::ProblemData;
use crate::utils::to_snake_case;

const INDOC_VERSION: &str = "2.0.6";
const DMOJ_VERSION: &str = "0.1.5";
const ASSERT_CMD_VERSION: &str = "2.0.17";

#[derive(Debug, Serialize)]
pub struct Sample {
    pub name: String,
    pub input: String,
    pub output: String,
    pub explanation: String,
    pub varnames: Vec<String>,
}

fn normalize_indent(s: &str) -> String {
    s.split('\n')
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_samples(
    inputs: &[String],
    outputs: &[String],
    explanations: &[String],
    varnames: &[Vec<String>],
) -> Vec<Sample> {
    inputs
        .iter()
        .zip(outputs.iter())
        .enumerate()
        .map(|(i, (inn, out))| Sample {
            name: if inputs.len() == 1 {
                "example".to_string()
            } else {
                format!("example_{}", i + 1)
            },
            input: normalize_indent(inn.trim()),
            output: normalize_indent(out.trim()),
            explanation: match explanations.get(i) {
                Some(e) if !e.is_empty() => {
                    textwrap::fill(&normalize_indent(e.trim()), 77) // max width = 88 - 9
                }
                _ => String::new(),
            },
            varnames: varnames.get(i).cloned().unwrap_or_default(),
        })
        .collect()
}

fn build_env(source: &str) -> Result<Environment<'static>> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    // the templates use Python string methods like `.splitlines()`
    env.set_unknown_method_callback(minijinja_contrib::pycompat::unknown_method_callback);

    match source {
        "dmoj" => {
            env.add_template(
                "Cargo.toml.j2",
                include_str!("../templates/dmoj/Cargo.toml.j2"),
            )?;
            env.add_template("main.rs.j2", include_str!("../templates/dmoj/main.rs.j2"))?;
            env.add_template("cli.rs.j2", include_str!("../templates/dmoj/cli.rs.j2"))?;
        }
        "leetcode" => {
            env.add_template(
                "Cargo.toml.j2",
                include_str!("../templates/leetcode/Cargo.toml.j2"),
            )?;
            env.add_template("lib.rs.j2", include_str!("../templates/leetcode/lib.rs.j2"))?;
        }
        _ => bail!("Unknown source: {source}"),
    }
    Ok(env)
}

/// Render the cargo project files for a scraped problem. Returns a list of
/// (relative path, contents) pairs.
pub fn render_rust_template(data: &ProblemData, source: &str) -> Result<Vec<(String, String)>> {
    let env = build_env(source)?;

    let mut fn_name = to_snake_case(&data.title);
    if let Some(sig) = &data.rust_signature {
        let parts: Vec<&str> = sig.split_whitespace().collect();
        if parts.len() >= 3 {
            fn_name = parts[2].split('(').next().unwrap_or("").to_string();
        }
    }

    let samples = format_samples(
        &data.sample_inputs,
        &data.sample_outputs,
        &data.sample_explanations,
        &data.sample_varnames,
    );

    match source {
        "leetcode" => {
            let lib_rs = env.get_template("lib.rs.j2")?.render(context! {
                title => &data.title,
                description => &data.description,
                constraints => &data.constraints,
                constraints_header => &data.constraints_header,
                input_header => data.input_header.as_deref().unwrap_or(""),
                input_spec => data.input_spec.as_deref().unwrap_or(""),
                output_header => data.output_header.as_deref().unwrap_or(""),
                output_spec => data.output_spec.as_deref().unwrap_or(""),
                function_name => &fn_name,
                rust_signature => &data.rust_signature,
                use_indoc => data.rust_signature.is_none(),
                samples => &samples,
            })?;
            let cargo_toml = env.get_template("Cargo.toml.j2")?.render(context! {
                name => to_snake_case(&data.title),
            })?;
            Ok(vec![
                ("src/lib.rs".to_string(), lib_rs),
                ("Cargo.toml".to_string(), cargo_toml),
            ])
        }
        "dmoj" => {
            let main_rs = env.get_template("main.rs.j2")?.render(context! {
                title => &data.title,
                description => &data.description,
                constraints => &data.constraints,
                constraints_header => &data.constraints_header,
                input_header => data.input_header.as_deref().unwrap_or(""),
                input_spec => data.input_spec.as_deref().unwrap_or(""),
                output_header => data.output_header.as_deref().unwrap_or(""),
                output_spec => data.output_spec.as_deref().unwrap_or(""),
            })?;
            let cli_rs = env.get_template("cli.rs.j2")?.render(context! {
                name => to_snake_case(&data.title),
                samples => &samples,
            })?;
            let cargo_toml = env.get_template("Cargo.toml.j2")?.render(context! {
                name => to_snake_case(&data.title),
                indoc_version => INDOC_VERSION,
                dmoj_version => DMOJ_VERSION,
                assert_cmd_version => ASSERT_CMD_VERSION,
            })?;
            Ok(vec![
                ("src/main.rs".to_string(), main_rs),
                ("tests/cli.rs".to_string(), cli_rs),
                ("Cargo.toml".to_string(), cargo_toml),
            ])
        }
        _ => bail!("Unknown source: {source}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dmoj_data() -> ProblemData {
        ProblemData {
            title: "A Plus B".to_string(),
            description: "Add two numbers.\n\nThat is all.".to_string(),
            constraints: Some("`1` <= `N` <= `10`^`6`".to_string()),
            constraints_header: Some("Constraints".to_string()),
            input_header: Some("Input Specification".to_string()),
            input_spec: Some("Two integers.".to_string()),
            output_header: Some("Output Specification".to_string()),
            output_spec: Some("Their sum.".to_string()),
            sample_inputs: vec!["1 2".to_string(), "3 4".to_string()],
            sample_outputs: vec!["3".to_string(), "7".to_string()],
            ..Default::default()
        }
    }

    fn leetcode_data() -> ProblemData {
        ProblemData {
            title: "Two Sum".to_string(),
            description: "Given an array of integers `nums`.".to_string(),
            constraints: Some("2 ≤ nums.length ≤ 10^4".to_string()),
            constraints_header: Some("Constraints".to_string()),
            sample_inputs: vec!["let nums = vec![2, 7, 11, 15];\nlet target = 9;".to_string()],
            sample_outputs: vec!["vec![0, 1]".to_string()],
            sample_explanations: vec!["Because nums[0] + nums[1] == 9.".to_string()],
            sample_varnames: vec![vec!["nums".to_string(), "target".to_string()]],
            rust_signature: Some(
                "pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {".to_string(),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn renders_dmoj_project() {
        let files = render_rust_template(&dmoj_data(), "dmoj").unwrap();
        let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, vec!["src/main.rs", "tests/cli.rs", "Cargo.toml"]);
        let joined = files
            .iter()
            .map(|(p, c)| format!("=== {p} ===\n{c}"))
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!("dmoj_project", joined);
    }

    #[test]
    fn renders_leetcode_project() {
        let files = render_rust_template(&leetcode_data(), "leetcode").unwrap();
        let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, vec!["src/lib.rs", "Cargo.toml"]);
        let joined = files
            .iter()
            .map(|(p, c)| format!("=== {p} ===\n{c}"))
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!("leetcode_project", joined);
    }

    #[test]
    fn unknown_source_errors() {
        let err = render_rust_template(&dmoj_data(), "codeforces").unwrap_err();
        assert_eq!(err.to_string(), "Unknown source: codeforces");
    }

    #[test]
    fn sample_naming() {
        let one = format_samples(&["a".into()], &["b".into()], &[], &[]);
        assert_eq!(one[0].name, "example");
        let two = format_samples(
            &["a".into(), "b".into()],
            &["c".into(), "d".into()],
            &[],
            &[],
        );
        assert_eq!(two[0].name, "example_1");
        assert_eq!(two[1].name, "example_2");
    }
}
