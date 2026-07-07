//! Pattern matcher: ties signatures, relevance, and the pattern library together.

use std::sync::Arc;

use nexus_cog_core::patterns::CodePattern;
use indexmap::IndexMap;
use parking_lot::RwLock;

use crate::builtin::builtin_patterns;
use crate::relevance::{RelevanceScorer, TaskHints};
use crate::signature::SignatureExtractor;

/// Pattern matcher with a built-in library and optional user-supplied patterns.
///
/// Cloning is cheap (Arc clone). Mutations use interior mutability via
/// [`parking_lot::RwLock`], so readers can hold their own snapshot without
/// lifetime ties.
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    patterns: IndexMap<String, CodePattern>,
    extractor: SignatureExtractor,
    scorer: RelevanceScorer,
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternMatcher {
    /// Construct a matcher preloaded with built-in patterns.
    #[must_use]
    pub fn new() -> Self {
        let mut patterns = IndexMap::new();
        for (id, pattern) in builtin_patterns() {
            patterns.insert(id.clone(), pattern.clone());
        }
        Self {
            inner: Arc::new(RwLock::new(Inner {
                patterns,
                extractor: SignatureExtractor::new(),
                scorer: RelevanceScorer::new(),
            })),
        }
    }

    /// Construct an empty matcher (no built-ins). Useful for tests.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                patterns: IndexMap::new(),
                extractor: SignatureExtractor::new(),
                scorer: RelevanceScorer::new(),
            })),
        }
    }

    /// Register a custom pattern. If a pattern with the same ID already exists, it is
    /// replaced.
    pub fn register(&self, pattern: CodePattern) {
        let mut inner = self.inner.write();
        inner.patterns.insert(pattern.id.clone(), pattern);
    }

    /// Unregister a pattern by ID.
    pub fn unregister(&self, id: &str) -> Option<CodePattern> {
        let mut inner = self.inner.write();
        inner.patterns.shift_remove(id)
    }

    /// Number of registered patterns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.read().patterns.len()
    }

    /// Returns `true` if no patterns are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.read().patterns.is_empty()
    }

    /// Returns all patterns (cloned).
    #[must_use]
    pub fn patterns(&self) -> Vec<CodePattern> {
        self.inner.read().patterns.values().cloned().collect()
    }

    /// Get a pattern by ID (cloned).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<CodePattern> {
        self.inner.read().patterns.get(id).cloned()
    }

    /// Find patterns that apply to a given piece of code.
    #[must_use]
    pub fn match_code(&self, code: &str, language: &str) -> Vec<CodePattern> {
        let inner = self.inner.read();
        let sigs = inner.extractor.extract(code);
        let lower = code.to_lowercase();
        inner
            .patterns
            .values()
            .filter(|p| {
                if p.context.language != "any" && p.context.language != language {
                    return false;
                }
                let sig_match = sigs.iter().any(|s| Self::signature_matches(p, s));
                let literal_match = lower.contains(&p.signature.to_lowercase());
                let example_match = p
                    .examples
                    .iter()
                    .any(|e| Self::code_contains_signature(code, e));
                sig_match || literal_match || example_match
            })
            .cloned()
            .collect()
    }

    /// Suggest the most relevant pattern for a task description.
    #[must_use]
    pub fn suggest_pattern(&self, task: &str, language: &str) -> Option<CodePattern> {
        let inner = self.inner.read();
        let hints = TaskHints {
            language: Some(language.to_string()),
            ..inner.scorer.extract_hints(task)
        };
        let patterns: Vec<CodePattern> = inner.patterns.values().cloned().collect();
        let ranked = inner.scorer.rank_owned(&patterns, &hints);
        ranked
            .into_iter()
            .find(|(p, s)| *s > 0.0 && (p.context.language == language || p.context.language == "any"))
            .map(|(p, _)| p)
    }

    /// Rank patterns by relevance to a task.
    #[must_use]
    pub fn rank_for_task(&self, task: &str) -> Vec<(CodePattern, f32)> {
        let inner = self.inner.read();
        let hints = inner.scorer.extract_hints(task);
        let patterns: Vec<CodePattern> = inner.patterns.values().cloned().collect();
        inner.scorer.rank_owned(&patterns, &hints)
    }

    /// Update a pattern's success rate based on observed outcome.
    pub fn record_outcome(&self, pattern_id: &str, success: bool) {
        let mut inner = self.inner.write();
        if let Some(p) = inner.patterns.get_mut(pattern_id) {
            p.match_count = p.match_count.saturating_add(1);
            const ALPHA: f32 = 0.1;
            p.success_rate = if success {
                p.success_rate * (1.0 - ALPHA) + ALPHA
            } else {
                p.success_rate * (1.0 - ALPHA)
            };
        }
    }

    fn signature_matches(
        pattern: &CodePattern,
        sig: &crate::signature::Signature,
    ) -> bool {
        use crate::signature::SignatureKind;
        match (&pattern.pattern_type, sig.kind) {
            (nexus_cog_core::patterns::PatternCategory::Async, SignatureKind::AsyncFunction) => true,
            (nexus_cog_core::patterns::PatternCategory::Async, SignatureKind::Function) => {
                sig.text.to_lowercase().contains("async")
            }
            (nexus_cog_core::patterns::PatternCategory::Builder, SignatureKind::Struct) => {
                sig.text.to_lowercase().contains("builder")
            }
            (nexus_cog_core::patterns::PatternCategory::Factory, SignatureKind::Function) => {
                let lower = sig.text.to_lowercase();
                lower.contains("create") || lower.contains("new") || lower.contains("make")
            }
            (nexus_cog_core::patterns::PatternCategory::StateMachine, SignatureKind::Enum) => {
                sig.text.to_lowercase().contains("state")
            }
            (nexus_cog_core::patterns::PatternCategory::ErrorHandling, _) => {
                sig.text.contains("Result") || sig.text.contains("Option")
            }
            _ => false,
        }
    }

    fn code_contains_signature(code: &str, example: &str) -> bool {
        let sig_words: Vec<&str> = example
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty() && w.len() > 3)
            .collect();
        sig_words.iter().any(|w| code.contains(w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_loads_builtin_patterns() {
        let m = PatternMatcher::new();
        assert!(!m.is_empty());
    }

    #[test]
    fn match_code_finds_rust_patterns() {
        let m = PatternMatcher::new();
        let code = "fn foo() -> Result<i32, E> { Ok(1) }";
        let matches = m.match_code(code, "rust");
        assert!(!matches.is_empty());
    }

    #[test]
    fn register_and_match() {
        let m = PatternMatcher::empty();
        let pattern = CodePattern::new(
            "my-pattern",
            nexus_cog_core::patterns::PatternCategory::Custom("test".to_string()),
            "specific_marker",
            0.9,
            nexus_cog_core::patterns::PatternContext::for_language("rust"),
        );
        m.register(pattern);
        let matches = m.match_code("let x = specific_marker;", "rust");
        assert!(!matches.is_empty());
    }

    #[test]
    fn record_outcome_updates_success_rate() {
        let m = PatternMatcher::empty();
        let pattern = CodePattern::new(
            "p",
            nexus_cog_core::patterns::PatternCategory::Custom("t".into()),
            "sig",
            0.5,
            nexus_cog_core::patterns::PatternContext::for_language("rust"),
        );
        m.register(pattern);
        m.record_outcome("p", true);
        let p = m.get("p").unwrap();
        assert!(p.success_rate > 0.5);
    }

    #[test]
    fn rank_for_task_returns_descending() {
        let m = PatternMatcher::new();
        let _ = m.rank_for_task("async tokio server");
    }

    #[test]
    fn unregister_removes_pattern() {
        let m = PatternMatcher::empty();
        let pattern = CodePattern::new(
            "to-remove",
            nexus_cog_core::patterns::PatternCategory::Custom("t".into()),
            "sig",
            0.5,
            nexus_cog_core::patterns::PatternContext::for_language("rust"),
        );
        m.register(pattern);
        assert!(m.get("to-remove").is_some());
        m.unregister("to-remove");
        assert!(m.get("to-remove").is_none());
    }

    #[test]
    fn clone_shares_state() {
        let m = PatternMatcher::empty();
        let pattern = CodePattern::new(
            "shared",
            nexus_cog_core::patterns::PatternCategory::Custom("t".into()),
            "sig",
            0.5,
            nexus_cog_core::patterns::PatternContext::for_language("rust"),
        );
        m.register(pattern);
        let m2 = m.clone();
        assert!(m2.get("shared").is_some());
    }
}
