use crate::{Document, Finding};

/// A pure, deterministic style-checking policy over one parsed [`Document`].
///
/// A rule has no failure mode of its own: given a `Document`, it always returns the (possibly
/// empty) set of findings, and calling it twice with the same document gives the same result.
pub trait Rule {
    /// Checks `doc` and returns every finding this rule produces, in no particular order (callers
    /// that need a specific order, such as the application layer's `(line, column)` sort, apply it
    /// themselves).
    fn check(&self, doc: &Document) -> Vec<Finding>;
}
