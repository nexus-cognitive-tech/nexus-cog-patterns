//! Relevance scoring: how well a pattern matches a given task description.
//!
//! Relevance combines keyword overlap, structural hints, and pattern success rate.
//! The score is in `[0.0, 1.0]`.

use nexus_cog_core::patterns::{CodePattern, PatternCategory};
use indexmap::IndexMap;

/// Hints extracted from a task description.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskHints {
    /// Programming language mentioned in the task.
    pub language: Option<String>,
    /// Framework mentioned (e.g. "tokio", "axum").
    pub framework: Option<String>,
    /// Keywords extracted from the task.
    pub keywords: Vec<String>,
    /// Category hints (e.g. `["error_handling", "async"]`).
    pub category_hints: Vec<PatternCategory>,
}

/// Scores patterns against a task description.
#[derive(Debug, Clone, Default)]
pub struct RelevanceScorer;

impl RelevanceScorer {
    /// Construct a new scorer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extract hints from a free-form task description.
    #[must_use]
    pub fn extract_hints(&self, task: &str) -> TaskHints {
        let lower = task.to_lowercase();
        let language = detect_language(&lower);
        let framework = detect_framework(&lower);
        let keywords = extract_keywords(&lower);
        let category_hints = detect_category_hints(&lower);
        TaskHints {
            language,
            framework,
            keywords,
            category_hints,
        }
    }

    /// Score a single pattern against the task hints.
    ///
    /// The score combines:
    /// - language match (0.3)
    /// - category hint match (0.3)
    /// - keyword overlap (0.3)
    /// - pattern success rate (0.1)
    #[must_use]
    pub fn score(&self, pattern: &CodePattern, hints: &TaskHints) -> f32 {
        let mut score = 0.0_f32;

        if let Some(lang) = &hints.language {
            if pattern.context.language == *lang {
                score += 0.3;
            }
        }
        if let Some(fw) = &hints.framework {
            if pattern.context.framework.as_deref() == Some(fw.as_str()) {
                score += 0.2;
            }
        }
        if !hints.category_hints.is_empty()
            && hints.category_hints.contains(&pattern.pattern_type)
        {
            score += 0.2;
        }

        let keyword_overlap = if !hints.keywords.is_empty() {
            let pat_keywords = keywords_for_category(&pattern.pattern_type);
            let hits = pat_keywords
                .iter()
                .filter(|k| hints.keywords.iter().any(|h| h == *k))
                .count();
            (hits as f32 / pat_keywords.len().max(1) as f32).min(1.0)
        } else {
            0.0
        };
        score += keyword_overlap * 0.2;

        score += pattern.success_rate * 0.1;

        score.clamp(0.0, 1.0)
    }

    /// Rank patterns by relevance to the task hints (descending).
    #[must_use]
    pub fn rank<'a, I>(&self, patterns: I, hints: &TaskHints) -> Vec<(&'a CodePattern, f32)>
    where
        I: IntoIterator<Item = &'a CodePattern>,
    {
        let mut scored: Vec<_> = patterns
            .into_iter()
            .map(|p| (p, self.score(p, hints)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    /// Rank owned patterns by relevance (descending).
    #[must_use]
    pub fn rank_owned(&self, patterns: &[CodePattern], hints: &TaskHints) -> Vec<(CodePattern, f32)> {
        let mut scored: Vec<_> = patterns
            .iter()
            .map(|p| (p.clone(), self.score(p, hints)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    /// Build a hint-to-pattern IndexMap for fast lookup.
    #[must_use]
    pub fn index<'a>(
        &self,
        patterns: &'a [CodePattern],
        hints: &TaskHints,
    ) -> IndexMap<String, &'a CodePattern> {
        self.rank(patterns, hints)
            .into_iter()
            .map(|(p, s)| (format!("{:.2}::{}", s, p.id), p))
            .collect()
    }
}

fn detect_language(lower: &str) -> Option<String> {
    let lang_keywords = [
        ("rust", "rust"),
        ("borrow", "rust"),
        ("crate", "rust"),
        ("tokio", "rust"),
        ("python", "python"),
        ("django", "python"),
        ("typescript", "typescript"),
        ("javascript", "javascript"),
        ("node", "javascript"),
        ("go ", "go"),
        ("golang", "go"),
        ("java ", "java"),
        ("kotlin", "kotlin"),
        ("swift", "swift"),
        ("c++", "cpp"),
    ];
    for (kw, lang) in lang_keywords {
        if lower.contains(kw) {
            return Some(lang.to_string());
        }
    }
    None
}

fn detect_framework(lower: &str) -> Option<String> {
    let frameworks = [
        "tokio", "axum", "actix", "rocket", "warp", "hyper", "reqwest", "serde",
        "diesel", "sqlx",
    ];
    frameworks
        .iter()
        .find(|f| lower.contains(*f))
        .map(|f| f.to_string())
}

fn extract_keywords(lower: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "in", "on", "at", "to", "for", "of", "and", "or", "is", "are",
        "be", "with", "as", "by", "this", "that", "it", "from", "use", "make", "create",
        "write", "implement", "add", "new", "function", "method", "class",
    ];
    lower
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(|w| w.to_string())
        .collect()
}

fn detect_category_hints(lower: &str) -> Vec<PatternCategory> {
    let mut hints = Vec::new();
    if lower.contains("error") || lower.contains("result") || lower.contains("exception") {
        hints.push(PatternCategory::ErrorHandling);
    }
    if lower.contains("async") || lower.contains("await") || lower.contains("concurrent") {
        hints.push(PatternCategory::Async);
    }
    if lower.contains("build") || lower.contains("construct") || lower.contains("configure") {
        hints.push(PatternCategory::Builder);
    }
    if lower.contains("factory") || lower.contains("create") || lower.contains("new") {
        hints.push(PatternCategory::Factory);
    }
    if lower.contains("observ") || lower.contains("subscribe") || lower.contains("notify") {
        hints.push(PatternCategory::Observer);
    }
    if lower.contains("state") && lower.contains("machine") {
        hints.push(PatternCategory::StateMachine);
    }
    if lower.contains("strategy") || lower.contains("algorithm") {
        hints.push(PatternCategory::Strategy);
    }
    if lower.contains("repository") || lower.contains("dao") || lower.contains("store") {
        hints.push(PatternCategory::Repository);
    }
    if lower.contains("decorat") || lower.contains("wrap") {
        hints.push(PatternCategory::Decorator);
    }
    if lower.contains("iter") || lower.contains("map") || lower.contains("fold") {
        hints.push(PatternCategory::Iterator);
    }
    if lower.contains("raii") || lower.contains("drop") || lower.contains("guard") {
        hints.push(PatternCategory::Raii);
    }
    hints
}

fn keywords_for_category(category: &PatternCategory) -> Vec<&'static str> {
    match category {
        PatternCategory::ErrorHandling => vec!["error", "result", "option", "handle", "parse", "panic"],
        PatternCategory::Async => vec!["async", "await", "tokio", "spawn", "future", "concurrent"],
        PatternCategory::Builder => vec!["builder", "construct", "create", "config", "chain"],
        PatternCategory::Factory => vec!["factory", "create", "new", "instantiate"],
        PatternCategory::Observer => vec!["observer", "subscribe", "publish", "notify", "listener"],
        PatternCategory::StateMachine => vec!["state", "machine", "transition", "finite"],
        PatternCategory::Strategy => vec!["strategy", "algorithm", "policy"],
        PatternCategory::Repository => vec!["repository", "dao", "store", "persist"],
        PatternCategory::Decorator => vec!["decorator", "wrap", "middleware", "intercept"],
        PatternCategory::Iterator => vec!["iter", "map", "fold", "collect", "filter"],
        PatternCategory::Raii => vec!["raii", "drop", "guard", "scope", "lifetime"],
        PatternCategory::Custom(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_cog_core::patterns::{CodePattern, PatternCategory, PatternComplexity, PatternContext};

    fn rust_pattern(id: &str) -> CodePattern {
        let mut p = CodePattern::new(
            id,
            PatternCategory::ErrorHandling,
            "-> Result<T, E>",
            0.9,
            PatternContext {
                language: "rust".to_string(),
                framework: None,
                complexity: PatternComplexity::Low,
                tags: vec![],
            },
        );
        p.description = "Propagate errors with Result and ?".into();
        p.examples = vec!["fn parse() -> Result<T, E> { Ok(x?) }".into()];
        p
    }

    #[test]
    fn language_detected_from_keywords() {
        let s = RelevanceScorer::new();
        let hints = s.extract_hints("implement a tokio async server in Rust");
        assert_eq!(hints.language.as_deref(), Some("rust"));
        assert_eq!(hints.framework.as_deref(), Some("tokio"));
        assert!(hints.keywords.contains(&"server".to_string()));
    }

    #[test]
    fn category_hints_detected() {
        let s = RelevanceScorer::new();
        let hints = s.extract_hints("add error handling for async parse function");
        assert!(hints.category_hints.contains(&PatternCategory::ErrorHandling));
        assert!(hints.category_hints.contains(&PatternCategory::Async));
    }

    #[test]
    fn rust_pattern_scores_high_on_rust_task() {
        let s = RelevanceScorer::new();
        let hints = s.extract_hints("implement error handling in tokio rust");
        let p = rust_pattern("rust-result");
        let score = s.score(&p, &hints);
        assert!(score > 0.5, "expected score > 0.5, got {score}");
    }

    #[test]
    fn rank_orders_descending() {
        let s = RelevanceScorer::new();
        let patterns = vec![
            {
                let mut p = rust_pattern("rust-error");
                p.success_rate = 0.5;
                p
            },
            {
                let mut p = rust_pattern("rust-result");
                p.success_rate = 0.95;
                p
            },
        ];
        let hints = s.extract_hints("rust error handling");
        let ranked = s.rank(&patterns, &hints);
        assert!(ranked[0].1 >= ranked[1].1);
    }
}
