use crate::GuildError;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Structured search result returned by knowledge search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeResult {
    pub title: String,
    pub snippet: String,
    pub relevance: f64,
}

/// Raw search result matching crosslink CLI JSON output.
#[derive(Deserialize)]
struct RawSearchResult {
    slug: String,
    #[allow(dead_code)]
    line_number: usize,
    context: Vec<RawSearchContext>,
}

/// Raw search result context line matching crosslink CLI JSON output.
#[derive(Deserialize)]
struct RawSearchContext {
    #[allow(dead_code)]
    line: usize,
    text: String,
}

/// Check if the crosslink CLI is installed and available in PATH.
pub fn crosslink_available() -> bool {
    Command::new("crosslink").arg("--version").output().is_ok()
}

/// Run a crosslink CLI command and return its stdout.
pub fn run(args: &[&str]) -> Result<String, GuildError> {
    let output = Command::new("crosslink")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GuildError::Crosslink(
                    "crosslink CLI is not installed or not found in PATH. Please install crosslink to use server-side features.".to_string()
                )
            } else {
                GuildError::Crosslink(format!("failed to spawn crosslink: {e}"))
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GuildError::Crosslink(format!(
            "crosslink {} failed: {}",
            args.first().unwrap_or(&""),
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Search the crosslink knowledge hub.
pub fn knowledge_search(query: &str) -> Result<Vec<KnowledgeResult>, GuildError> {
    let stdout = run(&["knowledge", "search", query, "--json"])?;
    let raw_results: Vec<RawSearchResult> = serde_json::from_str(&stdout).map_err(|e| {
        GuildError::Crosslink(format!("failed to parse crosslink JSON output: {e}"))
    })?;

    let results = raw_results
        .into_iter()
        .map(|raw| {
            let title = slug_to_title(&raw.slug);
            let snippet = raw
                .context
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            let relevance = calculate_relevance(query, &raw.slug, &snippet);

            KnowledgeResult {
                title,
                snippet,
                relevance,
            }
        })
        .collect();

    Ok(results)
}

fn slug_to_title(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn calculate_relevance(query: &str, slug: &str, snippet: &str) -> f64 {
    let query_lower = query.to_lowercase();
    let slug_lower = slug.to_lowercase();
    let snippet_lower = snippet.to_lowercase();

    let mut score = 0.0;

    // Exact phrase matches in slug/title or snippet
    if slug_lower.contains(&query_lower) {
        score += 10.0;
    }
    if snippet_lower.contains(&query_lower) {
        score += 5.0;
    }

    // Individual word occurrence counts
    for word in query_lower.split_whitespace() {
        if word.len() > 1 {
            score += snippet_lower.matches(word).count() as f64;
        }
    }

    score
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crosslink_available() {
        assert!(crosslink_available());
    }

    #[test]
    fn test_knowledge_search_returns_structured_results() {
        let results = knowledge_search("config").unwrap();
        assert!(
            !results.is_empty(),
            "expected some search results for 'config'"
        );

        for res in &results {
            assert!(!res.title.is_empty());
            assert!(!res.snippet.is_empty());
            assert!(res.relevance > 0.0);
        }
    }

    #[test]
    fn test_knowledge_search_empty() {
        let results = knowledge_search("nonexistent-search-query-uuid-123").unwrap();
        assert!(
            results.is_empty(),
            "expected no results for nonexistent query"
        );
    }
}
