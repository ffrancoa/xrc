use std::sync::LazyLock;

use anyhow::{Context, Result};
use regex::Regex;
use scraper::{ElementRef, Html, Node, Selector};

pub const MAX_WIDTH: usize = 84;

/// Equivalent of BeautifulSoup's `.text`: all descendant text nodes concatenated.
pub fn element_text(el: ElementRef) -> String {
    el.text().collect()
}

/// Equivalent of BeautifulSoup's `get_text(strip=True)`: each text node
/// stripped, empty ones dropped, joined without separator.
pub fn element_text_stripped(el: ElementRef) -> String {
    el.text().map(str::trim).filter(|s| !s.is_empty()).collect()
}

/// Equivalent of BeautifulSoup's `get_text("\n", strip=True)`.
pub fn element_text_stripped_joined(el: ElementRef) -> String {
    el.text()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_dmoj_text(text: &str) -> String {
    static TILDE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"~(.*?)~").unwrap());
    // `(?<!\w)([A-Za-z]\w*)(?!\w)` in Python; with a leading letter and greedy
    // `\w*` both lookarounds reduce to word boundaries.
    static IDENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b([A-Za-z]\w*)\b").unwrap());

    let replace_tilde_block = |caps: &regex::Captures| -> String {
        let content = caps[1]
            .replace(r"\le", "<=")
            .replace(r"\,", " ")
            .replace(r"\ne", "!=")
            .replace(r"\times", "×")
            .replace(r"\dots", "…");
        IDENT_RE.replace_all(&content, "`$1`").into_owned()
    };

    let mut processed_lines = Vec::new();
    for raw_line in text.split('\n') {
        let transformed = TILDE_RE.replace_all(raw_line.trim(), replace_tilde_block);
        let wrapped = if transformed.is_empty() {
            String::new()
        } else {
            textwrap::fill(&transformed, MAX_WIDTH)
        };
        processed_lines.push(wrapped);
    }
    processed_lines.join("\n")
}

fn process_code_content(s: &str) -> String {
    static CARET_WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\^\s*").unwrap());
    static WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
    // The trailing `(?!\w)` can follow `]`, where `\b` is not equivalent, so
    // this one needs real lookarounds (fancy-regex).
    static IDENT_RE: LazyLock<fancy_regex::Regex> = LazyLock::new(|| {
        fancy_regex::Regex::new(
            r"(?x)
            (?<!\w)                                     # no word char before
            ([A-Za-z]\w*(?:\[[^\]]+\]|\.[A-Za-z]\w*)*)  # base identifier
            (\^[A-Za-z]\w*)?                            # optional exponent
            (?!\w)                                      # no word char after
            ",
        )
        .unwrap()
    });

    let s = CARET_WS_RE.replace_all(s, "^");
    // only wrap the base in backticks, leave exponent raw
    let s = IDENT_RE.replace_all(&s, |caps: &fancy_regex::Captures| {
        let base = caps.get(1).map_or("", |m| m.as_str());
        let exp = caps.get(2).map_or("", |m| m.as_str());
        format!("`{base}`{exp}")
    });
    WS_RE.replace_all(s.trim(), " ").into_owned()
}

/// Text of a node with `<sup>n</sup>` descendants converted to `^n`.
fn collect_text_with_sup(node: ego_tree::NodeRef<Node>, out: &mut String) {
    if let Some(el) = ElementRef::wrap(node)
        && el.value().name() == "sup"
    {
        out.push('^');
        out.push_str(&element_text_stripped(el));
        return;
    }
    if let Node::Text(t) = node.value() {
        out.push_str(&t.text);
        return;
    }
    for child in node.children() {
        collect_text_with_sup(child, out);
    }
}

fn collect_leetcode_text(node: ego_tree::NodeRef<Node>, out: &mut String) {
    if let Some(el) = ElementRef::wrap(node) {
        match el.value().name() {
            "sup" => {
                out.push('^');
                out.push_str(&element_text_stripped(el));
                return;
            }
            "code" => {
                let mut inner = String::new();
                for child in node.children() {
                    collect_text_with_sup(child, &mut inner);
                }
                out.push_str(&process_code_content(&inner));
                return;
            }
            _ => {}
        }
    }
    if let Node::Text(t) = node.value() {
        out.push_str(&t.text);
        return;
    }
    for child in node.children() {
        collect_leetcode_text(child, out);
    }
}

/// Format small HTML snippets coming from LeetCode:
/// - convert `<sup>n</sup>` -> `^n` (without spaces)
/// - for `<code>...</code>` content, wrap only identifier-like tokens
///   (allowing dotted identifiers, e.g. `nums.length`) in backticks
/// - normalize whitespace and line-wrap to MAX_WIDTH
pub fn format_leetcode_text(html: &str) -> String {
    static LE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*≤\s*").unwrap());
    static GE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*≥\s*").unwrap());
    static WS_CARET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+\^").unwrap());
    static WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

    let doc = Html::parse_fragment(html);
    let mut text = String::new();
    for child in doc.tree.root().children() {
        collect_leetcode_text(child, &mut text);
    }

    // ensure spaces around ≤/≥ globally (just in case)
    let text = LE_RE.replace_all(&text, " ≤ ");
    let text = GE_RE.replace_all(&text, " ≥ ");

    // remove stray spaces before ^ and normalize spaces
    let text = WS_CARET_RE.replace_all(&text, "^");
    let text = WS_RE.replace_all(&text, " ");
    let text = text.trim();

    let mut processed_lines = Vec::new();
    for raw_line in text.split('\n') {
        let wrapped = if raw_line.trim().is_empty() {
            String::new()
        } else {
            textwrap::fill(raw_line.trim(), MAX_WIDTH)
        };
        processed_lines.push(wrapped);
    }
    processed_lines.join("\n")
}

/// Return true if the constraint looks like a mathematical expression.
pub fn is_math_constraint(line: &str) -> bool {
    static MATH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(≤|≥|<|>|=|\^\d|\d)").unwrap());
    MATH_RE.is_match(line)
}

/// Join constraints so that:
///   - Math constraints are grouped without blank lines.
///   - Exactly one blank line before the first textual constraint.
pub fn group_constraints(constraints: &[String]) -> String {
    let mut processed: Vec<String> = Vec::new();
    let mut last_was_math: Option<bool> = None;

    for line in constraints {
        let line = line.trim();
        if line.is_empty() {
            continue; // skip accidental empties
        }

        if is_math_constraint(line) {
            if last_was_math == Some(false) {
                processed.push(String::new()); // keep separation from previous text
            }
            processed.push(line.to_string());
            last_was_math = Some(true);
        } else {
            if last_was_math == Some(true) {
                processed.push(String::new()); // exactly one blank line before text
            }
            processed.push(line.to_string());
            last_was_math = Some(false);
        }
    }

    processed.join("\n")
}

pub fn to_snake_case(title: &str) -> String {
    static NON_WORD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^\w]+").unwrap());
    NON_WORD_RE
        .replace_all(&title.trim().to_lowercase(), "_")
        .trim_matches('_')
        .to_string()
}

pub fn extract_clean_title(doc: &Html) -> Result<String> {
    static H2_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h2").unwrap());
    let h2 = doc
        .select(&H2_SEL)
        .next()
        .context("No <h2> element found in the HTML content.")?;

    let raw_title = element_text_stripped(h2);
    if let Some((_, after)) = raw_title.split_once(" - ") {
        Ok(after.trim().to_string())
    } else {
        Ok(raw_title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dmoj_text_replaces_tilde_math() {
        // digits are not identifier-like, so they stay unwrapped (matches Python)
        assert_eq!(format_dmoj_text(r"~1 \le N \le 10^6~"), "1 <= `N` <= 10^6");
        assert_eq!(format_dmoj_text(r"~a \ne b~"), "`a` != `b`");
        assert_eq!(format_dmoj_text(r"~2 \times 3~"), "2 × 3");
        assert_eq!(format_dmoj_text(r"~x_1, \dots, x_n~"), "`x_1`, …, `x_n`");
    }

    #[test]
    fn dmoj_text_wraps_at_84() {
        let long = "word ".repeat(30);
        let out = format_dmoj_text(&long);
        assert!(out.lines().all(|l| l.chars().count() <= 84));
        assert!(out.lines().count() > 1);
    }

    #[test]
    fn dmoj_text_preserves_plain_lines() {
        assert_eq!(format_dmoj_text("hello world"), "hello world");
        assert_eq!(format_dmoj_text("  padded  "), "padded");
        assert_eq!(format_dmoj_text(""), "");
    }

    #[test]
    fn leetcode_text_converts_sup_and_code() {
        assert_eq!(format_leetcode_text("<p>10<sup>4</sup></p>"), "10^4");
        assert_eq!(
            format_leetcode_text("<p><code>nums.length</code></p>"),
            "`nums.length`"
        );
        assert_eq!(
            format_leetcode_text("<p><code>nums[i]</code></p>"),
            "`nums[i]`"
        );
        assert_eq!(
            format_leetcode_text("<p><code>2 <sup>10</sup></code></p>"),
            "2^10"
        );
    }

    #[test]
    fn leetcode_text_spaces_relational_ops() {
        assert_eq!(
            format_leetcode_text("<p>1≤n≤10<sup>4</sup></p>"),
            "1 ≤ n ≤ 10^4"
        );
    }

    #[test]
    fn math_constraint_detection() {
        assert!(is_math_constraint("1 ≤ n ≤ 10^4"));
        assert!(is_math_constraint("n > 0"));
        assert!(!is_math_constraint("only lowercase letters."));
    }

    #[test]
    fn group_constraints_inserts_single_blank_line() {
        let input = vec![
            "1 ≤ n ≤ 10".to_string(),
            "n is even".to_string(),
            "only letters.".to_string(),
        ];
        // "n is even" contains a digit-free... actually "n is even" has no digit,
        // but is_math_constraint checks for digits: none -> text.
        assert_eq!(
            group_constraints(&input),
            "1 ≤ n ≤ 10\n\nn is even\nonly letters."
        );
    }

    #[test]
    fn snake_case_conversion() {
        assert_eq!(to_snake_case("Two Sum"), "two_sum");
        assert_eq!(to_snake_case("  A+B Problem  "), "a_b_problem");
        assert_eq!(to_snake_case("Valid Parentheses!"), "valid_parentheses");
    }
}
