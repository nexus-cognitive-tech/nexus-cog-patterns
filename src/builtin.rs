//! Built-in pattern library.
//!
//! Curated patterns for common idioms. These are deliberately conservative — they
//! describe well-established practices, not novel tricks.

use nexus_cog_core::patterns::{CodePattern, PatternCategory, PatternComplexity, PatternContext};
use indexmap::IndexMap;
use std::sync::OnceLock;

/// Returns the full library of built-in patterns (lazily initialized).
#[must_use]
pub fn builtin_patterns() -> &'static IndexMap<String, CodePattern> {
    static LIB: OnceLock<IndexMap<String, CodePattern>> = OnceLock::new();
    LIB.get_or_init(build)
}

fn build() -> IndexMap<String, CodePattern> {
    let patterns: Vec<CodePattern> = vec![
        rust_result_propagation(),
        rust_async_tokio(),
        rust_option_combinators(),
        rust_builder(),
        rust_newtype(),
        rust_typestate(),
        ts_async_await(),
        ts_promise_all(),
        py_context_manager(),
        py_dataclass(),
        go_error_return(),
        go_goroutine_with_context(),
        java_try_with_resources(),
        cpp_raii(),
        observer_pattern(),
        strategy_pattern(),
        repository_pattern(),
        decorator_pattern(),
        factory_pattern(),
        iterator_pattern(),
    ];
    patterns.into_iter().map(|p| (p.id.clone(), p)).collect()
}

fn ctx(lang: &str, framework: Option<&str>, complexity: PatternComplexity) -> PatternContext {
    PatternContext {
        language: lang.to_string(),
        framework: framework.map(str::to_string),
        complexity,
        tags: vec![],
    }
}

fn rust_result_propagation() -> CodePattern {
    let mut p = CodePattern::new(
        "rust-result-propagation",
        PatternCategory::ErrorHandling,
        "fn -> Result<T, E>",
        0.95,
        ctx("rust", None, PatternComplexity::Low),
    );
    p.description = "Propagate errors with the `?` operator instead of `unwrap()`.".into();
    p.examples = vec![
        "fn parse(input: &str) -> Result<Data, Error> { let s = std::fs::read_to_string(input)?; Ok(parse_str(&s)) }".into(),
    ];
    p.anti_patterns = vec!["unwrap() on fallible operations".into()];
    p
}

fn rust_async_tokio() -> CodePattern {
    let mut p = CodePattern::new(
        "rust-async-tokio",
        PatternCategory::Async,
        "#[tokio::main] + async fn",
        0.92,
        ctx("rust", Some("tokio"), PatternComplexity::Medium),
    );
    p.description = "Use `#[tokio::main]` for the entrypoint and `async fn` for asynchronous operations.".into();
    p.examples = vec![
        "#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }".into(),
    ];
    p
}

fn rust_option_combinators() -> CodePattern {
    let mut p = CodePattern::new(
        "rust-option-combinators",
        PatternCategory::ErrorHandling,
        "Option::map | Option::and_then | Option::or_else",
        0.9,
        ctx("rust", None, PatternComplexity::Low),
    );
    p.description = "Use Option combinators (`map`, `and_then`, `or_else`) instead of `match` when chaining transformations.".into();
    p.examples = vec!["value.map(transform).and_then(fallback).unwrap_or(default)".into()];
    p
}

fn rust_builder() -> CodePattern {
    let mut p = CodePattern::new(
        "rust-builder",
        PatternCategory::Builder,
        "struct XBuilder { ... } with .field(..).build()",
        0.88,
        ctx("rust", None, PatternComplexity::Medium),
    );
    p.description = "Use the builder pattern for structs with many optional fields.".into();
    p.examples = vec![
        "Client::builder().timeout(Duration::from_secs(5)).build()".into(),
    ];
    p
}

fn rust_newtype() -> CodePattern {
    let mut p = CodePattern::new(
        "rust-newtype",
        PatternCategory::Custom("Newtype".into()),
        "struct Wrapper(Inner);",
        0.85,
        ctx("rust", None, PatternComplexity::Low),
    );
    p.description = "Wrap primitive types in a newtype to add type safety.".into();
    p.examples = vec!["struct UserId(u64);".into()];
    p
}

fn rust_typestate() -> CodePattern {
    let mut p = CodePattern::new(
        "rust-typestate",
        PatternCategory::Custom("Typestate".into()),
        "struct Locked; struct Unlocked;",
        0.78,
        ctx("rust", None, PatternComplexity::High),
    );
    p.description = "Encode state in the type system so invalid transitions become compile errors.".into();
    p.examples = vec![
        "fn consume(self: File<Open>) { ... }".into(),
    ];
    p
}

fn ts_async_await() -> CodePattern {
    let mut p = CodePattern::new(
        "ts-async-await",
        PatternCategory::Async,
        "async function + await",
        0.9,
        ctx("typescript", None, PatternComplexity::Low),
    );
    p.description = "Prefer `async/await` over raw Promise chains.".into();
    p.examples = vec!["const x = await fetch(url);".into()];
    p
}

fn ts_promise_all() -> CodePattern {
    let mut p = CodePattern::new(
        "ts-promise-all",
        PatternCategory::Async,
        "Promise.all([...])",
        0.88,
        ctx("typescript", None, PatternComplexity::Medium),
    );
    p.description = "Use `Promise.all` for parallel independent operations.".into();
    p.examples = vec!["const [a, b] = await Promise.all([fetchA(), fetchB()]);".into()];
    p
}

fn py_context_manager() -> CodePattern {
    let mut p = CodePattern::new(
        "py-context-manager",
        PatternCategory::Raii,
        "with open(...) as f:",
        0.92,
        ctx("python", None, PatternComplexity::Low),
    );
    p.description = "Use `with` statements for resource management.".into();
    p.examples = vec!["with open(path) as f: data = f.read()".into()];
    p
}

fn py_dataclass() -> CodePattern {
    let mut p = CodePattern::new(
        "py-dataclass",
        PatternCategory::Builder,
        "@dataclass",
        0.85,
        ctx("python", None, PatternComplexity::Low),
    );
    p.description = "Use `@dataclass` for value-like classes.".into();
    p.examples = vec!["@dataclass\nclass Point:\n    x: int\n    y: int".into()];
    p
}

fn go_error_return() -> CodePattern {
    let mut p = CodePattern::new(
        "go-error-return",
        PatternCategory::ErrorHandling,
        "func() (T, error)",
        0.95,
        ctx("go", None, PatternComplexity::Low),
    );
    p.description = "Return errors as values rather than panicking.".into();
    p.examples = vec!["func parse(s string) (int, error) { ... }".into()];
    p
}

fn go_goroutine_with_context() -> CodePattern {
    let mut p = CodePattern::new(
        "go-goroutine-context",
        PatternCategory::Async,
        "go func() { select { case <-ctx.Done(): ... } }",
        0.82,
        ctx("go", None, PatternComplexity::Medium),
    );
    p.description = "Pass a context.Context to goroutines so they can be cancelled.".into();
    p.examples = vec!["go worker(ctx)".into()];
    p
}

fn java_try_with_resources() -> CodePattern {
    let mut p = CodePattern::new(
        "java-try-with-resources",
        PatternCategory::Raii,
        "try (Resource r = ...) { ... }",
        0.93,
        ctx("java", None, PatternComplexity::Low),
    );
    p.description = "Use try-with-resources for AutoCloseable resources.".into();
    p.examples = vec!["try (var conn = ds.getConnection()) { ... }".into()];
    p
}

fn cpp_raii() -> CodePattern {
    let mut p = CodePattern::new(
        "cpp-raii",
        PatternCategory::Raii,
        "class Resource { ~Resource() { cleanup() } }",
        0.9,
        ctx("cpp", None, PatternComplexity::Medium),
    );
    p.description = "Tie resource lifetime to object lifetime via constructors/destructors.".into();
    p.examples = vec!["std::unique_ptr<T>".into()];
    p
}

fn observer_pattern() -> CodePattern {
    let mut p = CodePattern::new(
        "pattern-observer",
        PatternCategory::Observer,
        "trait Observer { fn update(&self, event: &Event); }",
        0.8,
        ctx("any", None, PatternComplexity::Medium),
    );
    p.description = "Decouple event producers from consumers via an Observer trait.".into();
    p.examples = vec!["emitter.subscribe(observer)".into()];
    p
}

fn strategy_pattern() -> CodePattern {
    let mut p = CodePattern::new(
        "pattern-strategy",
        PatternCategory::Strategy,
        "trait Strategy { fn execute(&self); }",
        0.82,
        ctx("any", None, PatternComplexity::Medium),
    );
    p.description = "Encapsulate algorithms behind a trait/interface so they can be swapped.".into();
    p.examples = vec!["let sorter: Box<dyn Sorter> = Box::new(QuickSorter);".into()];
    p
}

fn repository_pattern() -> CodePattern {
    let mut p = CodePattern::new(
        "pattern-repository",
        PatternCategory::Repository,
        "trait Repository { fn find(&self, id: Id) -> Option<T>; ... }",
        0.8,
        ctx("any", None, PatternComplexity::Medium),
    );
    p.description = "Hide data-access details behind a Repository trait.".into();
    p.examples = vec!["let user = repo.find(user_id)?;".into()];
    p
}

fn decorator_pattern() -> CodePattern {
    let mut p = CodePattern::new(
        "pattern-decorator",
        PatternCategory::Decorator,
        "struct Decorator { inner: T }",
        0.78,
        ctx("any", None, PatternComplexity::Medium),
    );
    p.description = "Add behavior by wrapping an object instead of subclassing.".into();
    p.examples = vec!["let logged = Logging::new(inner);".into()];
    p
}

fn factory_pattern() -> CodePattern {
    let mut p = CodePattern::new(
        "pattern-factory",
        PatternCategory::Factory,
        "fn create(kind: Kind) -> Box<dyn Product>",
        0.83,
        ctx("any", None, PatternComplexity::Low),
    );
    p.description = "Centralize object creation in a factory function.".into();
    p.examples = vec!["let p = factory.create(Kind::A);".into()];
    p
}

fn iterator_pattern() -> CodePattern {
    let mut p = CodePattern::new(
        "pattern-iterator",
        PatternCategory::Iterator,
        "trait Iterator { type Item; fn next(&mut self) -> Option<Self::Item>; }",
        0.86,
        ctx("any", None, PatternComplexity::Low),
    );
    p.description = "Provide sequential access without exposing the underlying representation.".into();
    p.examples = vec!["for x in iter { ... }".into()];
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_library_is_non_empty() {
        assert!(builtin_patterns().len() >= 10);
    }

    #[test]
    fn rust_patterns_present() {
        let lib = builtin_patterns();
        assert!(lib.contains_key("rust-result-propagation"));
        assert!(lib.contains_key("rust-async-tokio"));
    }

    #[test]
    fn cross_language_patterns_present() {
        let lib = builtin_patterns();
        assert!(lib.contains_key("pattern-observer"));
        assert!(lib.contains_key("pattern-strategy"));
    }
}
