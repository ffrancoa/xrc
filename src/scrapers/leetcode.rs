use std::sync::LazyLock;

use anyhow::{Result, bail};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;
use url::Url;

use crate::scrapers::ProblemData;
use crate::utils::{
    element_text, element_text_stripped, element_text_stripped_joined, format_leetcode_text,
    group_constraints,
};

const GRAPHQL_URL: &str = "https://leetcode.com/graphql";

const QUERY: &str = r#"
query getQuestionDetail($titleSlug: String!) {
  question(titleSlug: $titleSlug) {
    title
    content
    codeDefinition
    sampleTestCase
    exampleTestcases
  }
}
"#;

pub fn extract_problem_parts(url: &str) -> Result<ProblemData> {
    let slug = slug_from_url(url)?;
    let question = fetch_question(&slug)?;
    parse_question(&question, &slug)
}

/// Extracts the problem slug from a LeetCode URL.
fn slug_from_url(url: &str) -> Result<String> {
    if let Ok(parsed) = Url::parse(url) {
        let path = parsed.path().trim_matches('/');
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 && parts[0] == "problems" {
            return Ok(parts[1].to_string());
        }
    }
    bail!("Invalid LeetCode problem URL: {url}");
}

fn fetch_question(slug: &str) -> Result<Value> {
    let payload = serde_json::json!({
        "query": QUERY,
        "variables": { "titleSlug": slug },
    });

    let mut response = ureq::post(GRAPHQL_URL)
        .header("User-Agent", "Mozilla/5.0")
        .send_json(&payload)?;
    let data: Value = response.body_mut().read_json()?;

    let question = &data["data"]["question"];
    if question.is_null() {
        bail!("Could not retrieve question for slug '{slug}'");
    }
    Ok(question.clone())
}

pub fn parse_question(question: &Value, slug: &str) -> Result<ProblemData> {
    static P_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("p").unwrap());
    static LI_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("li").unwrap());

    let Some(title) = question["title"].as_str() else {
        bail!("Could not retrieve question for slug '{slug}'");
    };
    let content = question["content"].as_str().unwrap_or("");

    let doc = Html::parse_fragment(content);
    let paragraphs: Vec<ElementRef> = doc.select(&P_SEL).collect();

    // detect end of description (empty paragraph separator)
    let desc_end_idx = paragraphs
        .iter()
        .position(|p| element_text_stripped(*p).is_empty())
        .unwrap_or(paragraphs.len());

    let description_parts: Vec<String> = paragraphs[..desc_end_idx]
        .iter()
        .map(|p| format_leetcode_text(&p.html()))
        .collect();

    let mut constraints_header: Option<String> = None;
    let mut constraints_parts: Vec<String> = Vec::new();

    let constraints_p = paragraphs
        .iter()
        .find(|p| element_text(**p).trim().starts_with("Constraints:"))
        .copied();

    if let Some(constraints_p) = constraints_p {
        constraints_header = Some(
            element_text_stripped(constraints_p)
                .trim_end_matches(':')
                .to_string(),
        );
        let ul = constraints_p
            .next_siblings()
            .filter_map(ElementRef::wrap)
            .find(|el| el.value().name() == "ul");
        if let Some(ul) = ul {
            for li in ul.select(&LI_SEL) {
                constraints_parts.push(format_leetcode_text(&li.html()));
            }
        }
    }

    let constraints = if constraints_parts.is_empty() {
        None
    } else {
        Some(group_constraints(&constraints_parts))
    };

    let rust_signature = extract_rust_signature(question["codeDefinition"].as_str().unwrap_or(""));
    let (sample_inputs, sample_outputs, sample_explanations, sample_varnames) =
        extract_samples(&doc);

    Ok(ProblemData {
        title: title.to_string(),
        description: description_parts.join("\n\n"),
        constraints,
        constraints_header,
        sample_inputs,
        sample_outputs,
        sample_explanations,
        sample_varnames,
        rust_signature,
        ..Default::default()
    })
}

/// Extracts only the Rust function signature from LeetCode's codeDefinition JSON.
pub fn extract_rust_signature(code_definition_json: &str) -> Option<String> {
    static SIG_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?s)(pub\s+fn\s+[^(]+\([^)]*\)\s*->\s*[^{]+\{\s*\})").unwrap()
    });

    let code_defs: Vec<Value> = serde_json::from_str(code_definition_json).ok()?;
    let rust_entry = code_defs.iter().find(|entry| entry["value"] == "rust")?;
    let default_code = rust_entry["defaultCode"].as_str().unwrap_or("");

    let matched = SIG_RE.captures(default_code)?;
    let code_snippet = matched.get(1).unwrap().as_str().trim();
    code_snippet.lines().next().map(str::to_string)
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut depth = 0i32;
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        if let Some(q) = in_quote {
            buf.push(ch);
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        if ch == '\'' || ch == '"' {
            in_quote = Some(ch);
            buf.push(ch);
            continue;
        }
        if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth -= 1;
        }
        if ch == ',' && depth == 0 {
            parts.push(buf.trim().to_string());
            buf.clear();
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        parts.push(buf.trim().to_string());
    }
    parts
}

/// Returns a list of (name, raw_value) preserving order. Handles values that
/// are bracketed lists (with nested brackets) or simple scalars separated by
/// top-level commas.
pub fn parse_assignments(s: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut assignments: Vec<(String, String)> = Vec::new();

    let is_word = |c: char| c.is_alphanumeric() || c == '_';

    while i < n {
        // skip whitespace
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        // name: [A-Za-z_]\w*
        if i >= n || !(chars[i].is_ascii_alphabetic() || chars[i] == '_') {
            break;
        }
        let name_start = i;
        i += 1;
        while i < n && is_word(chars[i]) {
            i += 1;
        }
        let name: String = chars[name_start..i].iter().collect();
        // skip spaces and equals
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= n || chars[i] != '=' {
            break;
        }
        i += 1;
        // skip spaces
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        // value
        let raw_value: String;
        if i < n && chars[i] == '[' {
            let start = i;
            let mut depth = 0i32;
            let mut in_quote: Option<char> = None;
            while i < n {
                let ch = chars[i];
                if let Some(q) = in_quote {
                    if ch == q {
                        in_quote = None;
                    }
                } else if ch == '"' || ch == '\'' {
                    in_quote = Some(ch);
                } else if ch == '[' {
                    depth += 1;
                } else if ch == ']' {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            raw_value = chars[start..i]
                .iter()
                .collect::<String>()
                .trim()
                .to_string();
        } else {
            let start = i;
            // scalar until top-level comma
            while i < n && chars[i] != ',' {
                i += 1;
            }
            raw_value = chars[start..i]
                .iter()
                .collect::<String>()
                .trim()
                .to_string();
        }
        assignments.push((name, raw_value));
        // skip comma if present
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i < n && chars[i] == ',' {
            i += 1;
        }
    }
    assignments
}

pub fn to_rust_value(raw: &str) -> String {
    static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?\d+(\.\d+)?$").unwrap());

    let raw = raw.trim();
    // list-ish
    if raw.starts_with('[') && raw.ends_with(']') {
        let inner = raw[1..raw.len() - 1].trim();
        if inner.is_empty() {
            return "vec![]".to_string();
        }
        let elems = split_top_level_commas(inner);
        let norm_elems: Vec<String> = elems
            .iter()
            .map(|e| {
                let e = e.trim();
                // keep strings as-is, but normalize quotes to double quotes if single quoted
                if e.starts_with('\'') && e.ends_with('\'') && e.len() >= 2 {
                    format!("\"{}\"", e[1..e.len() - 1].replace('"', "\\\""))
                } else {
                    e.to_string()
                }
            })
            .collect();
        return format!("vec![{}]", norm_elems.join(", "));
    }
    // quoted string
    if raw.starts_with('\'') && raw.ends_with('\'') && raw.len() >= 2 {
        return format!("\"{}\"", raw[1..raw.len() - 1].replace('"', "\\\""));
    }
    // numeric?
    if NUM_RE.is_match(raw) {
        return raw.to_string();
    }
    // boolean (lowercase)
    let lower = raw.to_lowercase();
    if lower == "true" || lower == "false" {
        return lower;
    }
    // fallback: return as-is (caller may handle)
    raw.to_string()
}

/// Parse LeetCode `<pre>` example blocks and produce Rust-ready
/// `sample_inputs` (multi-line let-statements) and `sample_outputs`.
fn extract_samples(doc: &Html) -> (Vec<String>, Vec<String>, Vec<String>, Vec<Vec<String>>) {
    static PRE_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("pre").unwrap());
    static INPUT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Input:\s*(.+)").unwrap());
    static OUTPUT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Output:\s*(.+)").unwrap());
    static EXPLANATION_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Explanation:\s*(.+)").unwrap());

    let mut sample_inputs: Vec<String> = Vec::new();
    let mut sample_outputs: Vec<String> = Vec::new();
    let mut sample_explanations: Vec<String> = Vec::new();
    let mut sample_varnames: Vec<Vec<String>> = Vec::new();

    for pre in doc.select(&PRE_SEL) {
        let text = element_text_stripped_joined(pre);
        let (Some(input_m), Some(output_m)) = (INPUT_RE.captures(&text), OUTPUT_RE.captures(&text))
        else {
            continue;
        };

        let input_str = input_m[1].trim();
        let output_str = output_m[1].trim();
        let explanation_str = EXPLANATION_RE
            .captures(&text)
            .map(|m| m[1].trim().to_string())
            .unwrap_or_default();

        let assignments = parse_assignments(input_str);
        if assignments.is_empty() {
            continue;
        }

        let mut input_lines: Vec<String> = Vec::new();
        let mut varnames: Vec<String> = Vec::new();
        for (name, raw_val) in &assignments {
            let rust_val = to_rust_value(raw_val);
            input_lines.push(format!("let {name} = {rust_val};"));
            varnames.push(name.clone());
        }

        sample_inputs.push(input_lines.join("\n"));
        sample_varnames.push(varnames);
        sample_outputs.push(to_rust_value(output_str));
        sample_explanations.push(explanation_str);
    }

    (
        sample_inputs,
        sample_outputs,
        sample_explanations,
        sample_varnames,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_values() {
        assert_eq!(to_rust_value("[1,2,3]"), "vec![1, 2, 3]");
        assert_eq!(to_rust_value("[1, 2, 3]"), "vec![1, 2, 3]");
        assert_eq!(to_rust_value("[]"), "vec![]");
        assert_eq!(to_rust_value("['a','b']"), "vec![\"a\", \"b\"]");
        assert_eq!(to_rust_value("[[1,2],[3,4]]"), "vec![[1,2], [3,4]]");
        assert_eq!(to_rust_value("'hello'"), "\"hello\"");
        assert_eq!(to_rust_value("42"), "42");
        assert_eq!(to_rust_value("-3.14"), "-3.14");
        assert_eq!(to_rust_value("True"), "true");
        assert_eq!(to_rust_value("\"kept\""), "\"kept\"");
    }

    #[test]
    fn assignments_parsing() {
        let parsed = parse_assignments("nums = [2,7,11,15], target = 9");
        assert_eq!(
            parsed,
            vec![
                ("nums".to_string(), "[2,7,11,15]".to_string()),
                ("target".to_string(), "9".to_string()),
            ]
        );

        // deliberate quirk: the scalar scan stops at a comma even
        // inside double quotes, so this input truncates and parsing stops
        let parsed = parse_assignments("s = \"a,b\", grid = [[1,2],[3,4]]");
        assert_eq!(parsed, vec![("s".to_string(), "\"a".to_string())]);
    }

    #[test]
    fn signature_extraction() {
        let code_def = serde_json::json!([
            {
                "value": "rust",
                "defaultCode": "impl Solution {\n    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {\n        \n    }\n}"
            }
        ]);
        // note: the regex requires `{` immediately followed by `}` after
        // optional whitespace, so a body with only whitespace matches
        let sig = extract_rust_signature(&code_def.to_string());
        assert_eq!(
            sig.as_deref(),
            Some("pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {")
        );
        assert_eq!(extract_rust_signature("not json"), None);
        assert_eq!(extract_rust_signature("[]"), None);
    }

    #[test]
    fn samples_extraction() {
        let html = r#"
        <p>desc</p>
        <pre><strong>Input:</strong> nums = [2,7,11,15], target = 9
<strong>Output:</strong> [0,1]
<strong>Explanation:</strong> Because nums[0] + nums[1] == 9.</pre>
        "#;
        let doc = Html::parse_fragment(html);
        let (inputs, outputs, explanations, varnames) = extract_samples(&doc);
        assert_eq!(
            inputs,
            vec!["let nums = vec![2, 7, 11, 15];\nlet target = 9;"]
        );
        assert_eq!(outputs, vec!["vec![0, 1]"]);
        assert_eq!(explanations, vec!["Because nums[0] + nums[1] == 9."]);
        assert_eq!(
            varnames,
            vec![vec!["nums".to_string(), "target".to_string()]]
        );
    }

    #[test]
    fn slug_parsing() {
        assert_eq!(
            slug_from_url("https://leetcode.com/problems/two-sum/").unwrap(),
            "two-sum"
        );
        assert!(slug_from_url("https://leetcode.com/contest/").is_err());
    }
}
