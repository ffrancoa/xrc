pub mod dmoj;
pub mod leetcode;

use anyhow::Result;
use url::Url;

/// The contract between a scraper and the renderer, mirroring the dict
/// returned by the Python `extract_problem_parts` functions.
#[derive(Debug, Default)]
pub struct ProblemData {
    pub title: String,
    pub description: String,
    pub constraints: Option<String>,
    pub constraints_header: Option<String>,
    pub input_header: Option<String>,
    pub input_spec: Option<String>,
    pub output_header: Option<String>,
    pub output_spec: Option<String>,
    pub sample_inputs: Vec<String>,
    pub sample_outputs: Vec<String>,
    pub sample_explanations: Vec<String>,
    pub sample_varnames: Vec<Vec<String>>,
    pub rust_signature: Option<String>,
}

type ScraperFn = fn(&str) -> Result<ProblemData>;

const SCRAPERS: &[(&str, ScraperFn)] = &[
    ("dmoj.ca", dmoj::extract_problem_parts),
    ("leetcode.com", leetcode::extract_problem_parts),
];

/// Equivalent of Python's `urlparse(url).netloc` (host plus optional port).
pub fn netloc(url: &str) -> String {
    let Ok(parsed) = Url::parse(url) else {
        return String::new();
    };
    let host = parsed.host_str().unwrap_or("");
    match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    }
}

/// Find the appropriate scraper for a given URL. The source name is the
/// first label of the registered host (e.g. "dmoj", "leetcode").
pub fn find_scraper(url: &str) -> Option<(&'static str, ScraperFn)> {
    let netloc = netloc(url);
    for (host, scraper_fn) in SCRAPERS {
        if netloc.contains(host) {
            let source = host.split('.').next().unwrap_or(host);
            return Some((source, *scraper_fn));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_registered_scrapers() {
        let (source, _) = find_scraper("https://dmoj.ca/problem/aplusb").unwrap();
        assert_eq!(source, "dmoj");
        let (source, _) = find_scraper("https://leetcode.com/problems/two-sum/").unwrap();
        assert_eq!(source, "leetcode");
        // substring match on the netloc, like the Python version
        let (source, _) = find_scraper("https://www.dmoj.ca/problem/aplusb").unwrap();
        assert_eq!(source, "dmoj");
    }

    #[test]
    fn rejects_unknown_sites() {
        assert!(find_scraper("https://example.com/problem/1").is_none());
        assert!(find_scraper("not a url").is_none());
    }

    #[test]
    fn netloc_extraction() {
        assert_eq!(netloc("https://dmoj.ca/problem/x"), "dmoj.ca");
        assert_eq!(netloc("https://dmoj.ca:8080/problem/x"), "dmoj.ca:8080");
        assert_eq!(netloc("garbage"), "");
    }
}
