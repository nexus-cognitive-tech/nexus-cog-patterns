//! Pattern library and matcher.
//!
//! Patterns describe reusable idioms that the agent should reach for (e.g. `Result`
//! propagation, async with Tokio, builder pattern). The library ships with built-in
//! patterns for popular languages and lets the caller register custom ones.
//!
//! # Layers
//!
//! - [`builtin`] — the default pattern set, lazily initialized.
//! - [`signature`] — lightweight signature extraction for matching.
//! - [`relevance`] — relevance scoring (how well a pattern matches a task).
//! - [`matcher`] — the public [`PatternMatcher`] that ties it all together.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod builtin;
pub mod matcher;
pub mod relevance;
pub mod signature;

pub use matcher::PatternMatcher;
pub use relevance::{RelevanceScorer, TaskHints};
pub use signature::SignatureExtractor;
