//! `lint-domain`: value objects, the [`Document`] type, the [`Rule`] trait, and the MD009/MD047
//! rule implementations for `lint` (ADR-0002).
//!
//! This crate has no dependency on I/O or a framework: no `pulldown-cmark`, no `clap`, no
//! `std::fs`, no `std::env`. Everything here is a pure function over already-loaded data, so it is
//! unit-testable and reusable without pulling in a file system or a CLI parser.

mod document;
mod finding;
mod location;
mod rule;
mod rule_id;
pub mod rules;

pub use document::Document;
pub use finding::Finding;
pub use location::{Location, LocationError};
pub use rule::Rule;
pub use rule_id::RuleId;
pub use rules::RULES;
