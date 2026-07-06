use std::sync::LazyLock;

use anyhow::{Result, bail};
use scraper::{ElementRef, Html, Selector};

use crate::scrapers::ProblemData;
use crate::utils::{element_text, extract_clean_title, format_dmoj_text};

pub fn extract_problem_parts(url: &str) -> Result<ProblemData> {
    let html = fetch(url)?;
    parse(&html)
}

fn fetch(url: &str) -> Result<String> {
    match fetch_direct(url) {
        Ok(html) => Ok(html),
        Err(e) if e.to_string().contains("403") => {
            eprintln!("blocked by cloudflare, retrying via web archive...");
            fetch_web_archive(url)
        }
        Err(e) => Err(e),
    }
}

fn http_get(url: &str) -> Result<String> {
    let mut response = ureq::get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:139.0) Gecko/20100101 Firefox/139.0",
        )
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.5")
        .call()?;
    Ok(response.body_mut().read_to_string()?)
}

fn fetch_direct(url: &str) -> Result<String> {
    http_get(url)
}

fn fetch_web_archive(url: &str) -> Result<String> {
    let archive_url = format!("https://web.archive.org/web/2024/{url}");
    http_get(&archive_url)
}

fn next_sibling_elements(el: ElementRef<'_>) -> impl Iterator<Item = ElementRef<'_>> {
    el.next_siblings().filter_map(ElementRef::wrap)
}

pub fn parse(html: &str) -> Result<ProblemData> {
    static H4_SEL: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h4").unwrap());

    let doc = Html::parse_document(html);
    let title = extract_clean_title(&doc)?;

    let h4_tags: Vec<ElementRef> = doc.select(&H4_SEL).collect();

    let find_h4 = |text: &str| {
        h4_tags
            .iter()
            .find(|h| element_text(**h).trim() == text)
            .copied()
    };
    let constraints_h4 = find_h4("Constraints");
    let input_h4 = find_h4("Input Specification");
    let output_h4 = find_h4("Output Specification");

    let (Some(input_h4), Some(output_h4)) = (input_h4, output_h4) else {
        bail!("Could not find all required section headers.");
    };

    let first_h4 = h4_tags[0];

    // Equivalent of BS4 `first_h4.find_all_previous()`: elements before
    // first_h4 in reverse document order, collecting <p> until an <h2>.
    let mut preceding: Vec<ElementRef> = Vec::new();
    for node in doc.tree.root().descendants() {
        if node.id() == first_h4.id() {
            break;
        }
        if let Some(el) = ElementRef::wrap(node) {
            preceding.push(el);
        }
    }
    let mut description_parts: Vec<String> = Vec::new();
    for el in preceding.iter().rev() {
        match el.value().name() {
            "h2" => break,
            "p" => description_parts.insert(0, element_text(*el).trim().to_string()),
            _ => {}
        }
    }

    let mut constraints_parts: Vec<String> = Vec::new();
    if let Some(constraints_h4) = constraints_h4 {
        for tag in next_sibling_elements(constraints_h4) {
            if tag.id() == input_h4.id() {
                break;
            }
            if tag.value().name() == "p" {
                constraints_parts.push(element_text(tag).trim().to_string());
            }
        }
    }

    let mut input_parts: Vec<String> = Vec::new();
    for tag in next_sibling_elements(input_h4) {
        if tag.id() == output_h4.id() {
            break;
        }
        if tag.value().name() == "p" {
            input_parts.push(element_text(tag).trim().to_string());
        }
    }

    let mut output_parts: Vec<String> = Vec::new();
    for tag in next_sibling_elements(output_h4) {
        if tag.value().name() == "h4" && element_text(tag).trim().starts_with("Sample Input") {
            break;
        }
        if tag.value().name() == "p" {
            output_parts.push(element_text(tag).trim().to_string());
        }
    }

    let first_pre_after = |h4: ElementRef| -> Option<String> {
        next_sibling_elements(h4)
            .find(|tag| tag.value().name() == "pre")
            .map(|pre| element_text(pre).trim().to_string())
    };

    let mut sample_inputs: Vec<String> = Vec::new();
    let mut sample_outputs: Vec<String> = Vec::new();
    for h in &h4_tags {
        let heading = element_text(*h);
        let heading = heading.trim();
        if heading.starts_with("Sample Input")
            && let Some(pre) = first_pre_after(*h)
        {
            sample_inputs.push(pre);
        } else if heading.starts_with("Sample Output")
            && let Some(pre) = first_pre_after(*h)
        {
            sample_outputs.push(pre);
        }
    }

    let join_formatted = |parts: &[String]| -> String {
        parts
            .iter()
            .map(|p| format_dmoj_text(p))
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    Ok(ProblemData {
        title,
        description: join_formatted(&description_parts),
        constraints: if constraints_parts.is_empty() {
            None
        } else {
            Some(join_formatted(&constraints_parts))
        },
        input_spec: Some(join_formatted(&input_parts)),
        output_spec: Some(join_formatted(&output_parts)),
        constraints_header: constraints_h4.map(|h| element_text(h).trim().to_string()),
        input_header: Some(element_text(input_h4).trim().to_string()),
        output_header: Some(element_text(output_h4).trim().to_string()),
        sample_inputs,
        sample_outputs,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_PAGE: &str = r##"
    <html><body>
      <h2>APIO '10 P1 - Commando</h2>
      <p>You are the commander of ~n~ soldiers.</p>
      <h4>Constraints</h4>
      <p>~1 \le n \le 10~</p>
      <h4>Input Specification</h4>
      <p>The first line contains ~n~.</p>
      <h4>Output Specification</h4>
      <p>Print one integer.</p>
      <h4>Sample Input</h4>
      <pre>3</pre>
      <h4>Sample Output</h4>
      <pre>9</pre>
    </body></html>
    "##;

    #[test]
    fn parses_minimal_page() {
        let data = parse(MINIMAL_PAGE).unwrap();
        assert_eq!(data.title, "Commando");
        assert_eq!(data.description, "You are the commander of `n` soldiers.");
        assert_eq!(data.constraints.as_deref(), Some("1 <= `n` <= 10"));
        assert_eq!(data.constraints_header.as_deref(), Some("Constraints"));
        assert_eq!(data.input_header.as_deref(), Some("Input Specification"));
        assert_eq!(
            data.input_spec.as_deref(),
            Some("The first line contains `n`.")
        );
        assert_eq!(data.output_spec.as_deref(), Some("Print one integer."));
        assert_eq!(data.sample_inputs, vec!["3"]);
        assert_eq!(data.sample_outputs, vec!["9"]);
        assert!(data.rust_signature.is_none());
    }

    #[test]
    fn missing_sections_error() {
        let err = parse("<html><h2>X - Y</h2><h4>Other</h4></html>").unwrap_err();
        assert_eq!(
            err.to_string(),
            "Could not find all required section headers."
        );
    }
}
