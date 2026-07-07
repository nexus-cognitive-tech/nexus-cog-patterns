//! Signature extraction utilities.
//!
//! Signatures are short structural fingerprints of source code used for fast pattern
//! matching without a full parser. They are intentionally lossy — the goal is to
//! recognize *categories* of code, not exact syntax.

use nexus_cog_core::Language;

/// Extracts lightweight signatures from source code.
#[derive(Debug, Clone, Default)]
pub struct SignatureExtractor;

impl SignatureExtractor {
    /// Construct a new extractor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extract all signatures present in the code, preserving order.
    #[must_use]
    pub fn extract(&self, code: &str) -> Vec<Signature> {
        let mut out = Vec::new();
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim_start();
            if let Some(sig) = self.classify_line(trimmed) {
                out.push(Signature {
                    line: (i + 1) as u32,
                    kind: sig,
                    text: trimmed.to_string(),
                });
            }
        }
        out
    }

    /// Returns `true` if the code contains a function definition.
    #[must_use]
    pub fn has_function(&self, code: &str) -> bool {
        code.contains("fn ")
            || code.contains("function ")
            || code.contains("def ")
            || code.contains("func ")
    }

    /// Returns `true` if the code contains a type definition.
    #[must_use]
    pub fn has_type_def(&self, code: &str) -> bool {
        code.contains("struct ")
            || code.contains("enum ")
            || code.contains("trait ")
            || code.contains("interface ")
            || code.contains("class ")
    }

    /// Returns `true` if the code contains an `async` construct.
    #[must_use]
    pub fn has_async(&self, code: &str) -> bool {
        code.contains("async ") || code.contains("async\n") || code.contains("async{")
    }

    /// Returns `true` if the code uses `Result` or `?`.
    #[must_use]
    pub fn uses_result(&self, code: &str) -> bool {
        code.contains("Result<") || code.contains("Ok(") || code.contains('?')
    }

    /// Returns `true` if the code uses `Option`.
    #[must_use]
    pub fn uses_option(&self, code: &str) -> bool {
        code.contains("Option<") || code.contains("Some(") || code.contains("None")
    }

    /// Returns `true` if the code contains unsafe code (Rust-specific).
    #[must_use]
    pub fn has_unsafe(&self, code: &str, language: Language) -> bool {
        match language {
            Language::Rust => code.contains("unsafe "),
            _ => false,
        }
    }

    /// Count function definitions.
    #[must_use]
    pub fn count_functions(&self, code: &str) -> usize {
        let mut count = 0;
        let bytes = code.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            // Look for "fn " not preceded by "uncti" (avoids counting inside "function ").
            if i + 3 <= len && &bytes[i..i + 3] == b"fn " {
                if i >= 5 && &bytes[i - 5..i] == b"uncti" {
                    i += 3;
                    continue;
                }
                count += 1;
                i += 3;
                continue;
            }
            // Look for "def ".
            if i + 4 <= len && &bytes[i..i + 4] == b"def " {
                count += 1;
                i += 4;
                continue;
            }
            // Look for "func " not preceded by "ture ".
            if i + 5 <= len && &bytes[i..i + 5] == b"func " {
                if i >= 5 && &bytes[i - 5..i] == b"ture " {
                    i += 5;
                    continue;
                }
                count += 1;
                i += 5;
                continue;
            }
            i += 1;
        }
        count
    }

    fn classify_line(&self, trimmed: &str) -> Option<SignatureKind> {
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            Some(SignatureKind::Function)
        } else if trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
            Some(SignatureKind::AsyncFunction)
        } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
            Some(SignatureKind::Struct)
        } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
            Some(SignatureKind::Enum)
        } else if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
            Some(SignatureKind::Trait)
        } else if trimmed.starts_with("impl ") {
            Some(SignatureKind::Impl)
        } else if trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ") {
            Some(SignatureKind::Module)
        } else if trimmed.starts_with("unsafe ") {
            Some(SignatureKind::Unsafe)
        } else if trimmed.starts_with("macro_rules!") || trimmed.starts_with("#[macro_export]") {
            Some(SignatureKind::Macro)
        } else if trimmed.starts_with("use ") {
            Some(SignatureKind::Import)
        } else {
            None
        }
    }
}

/// A single extracted signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// 1-indexed line.
    pub line: u32,
    /// Signature kind.
    pub kind: SignatureKind,
    /// Original line (trimmed).
    pub text: String,
}

/// Kind of signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignatureKind {
    /// Regular function.
    Function,
    /// Async function.
    AsyncFunction,
    /// Struct definition.
    Struct,
    /// Enum definition.
    Enum,
    /// Trait definition.
    Trait,
    /// Impl block.
    Impl,
    /// Module declaration.
    Module,
    /// Unsafe block.
    Unsafe,
    /// Macro definition.
    Macro,
    /// Use statement / import.
    Import,
}

impl SignatureKind {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::AsyncFunction => "async_function",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::Module => "module",
            Self::Unsafe => "unsafe",
            Self::Macro => "macro",
            Self::Import => "import",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rust_signatures() {
        let code = r#"
            use std::io;
            pub struct Foo { x: i32 }
            impl Foo {
                pub fn new() -> Self { Self { x: 0 } }
                pub async fn run(&self) -> Result<(), io::Error> { Ok(()) }
            }
            unsafe fn danger() {}
            trait Greet { fn hi(&self); }
            macro_rules! my_macro { () => {}; }
            pub mod inner;
        "#;
        let sigs = SignatureExtractor::new().extract(code);
        let kinds: Vec<_> = sigs.iter().map(|s| s.kind.id()).collect();
        assert!(kinds.contains(&"import"));
        assert!(kinds.contains(&"struct"));
        assert!(kinds.contains(&"impl"));
        assert!(kinds.contains(&"function"));
        assert!(kinds.contains(&"async_function"));
        assert!(kinds.contains(&"unsafe"));
        assert!(kinds.contains(&"trait"));
        assert!(kinds.contains(&"macro"));
        assert!(kinds.contains(&"module"));
    }

    #[test]
    fn helper_flags_work() {
        let e = SignatureExtractor::new();
        let code = "fn foo() -> Result<i32, E> { let x = bar()?; Ok(x) }";
        assert!(e.has_function(code));
        assert!(e.uses_result(code));
        assert!(!e.has_async(code));
        let code2 = "async fn foo() {}";
        assert!(e.has_async(code2));
    }

    #[test]
    fn function_count_works() {
        let e = SignatureExtractor::new();
        assert_eq!(e.count_functions("fn a() {} fn b() {} fn c() {}"), 3);
        assert_eq!(e.count_functions("def x(): pass\ndef y(): pass"), 2);
    }

    #[test]
    fn has_unsafe_only_for_rust() {
        let e = SignatureExtractor::new();
        assert!(e.has_unsafe("unsafe fn x() {}", Language::Rust));
        assert!(!e.has_unsafe("unsafe fn x() {}", Language::TypeScript));
    }
}
