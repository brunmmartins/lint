//! The rule implementations in scope for LINT-001, and the static registry the engine iterates.

mod md009;
mod md047;

pub use md009::Md009;
pub use md047::Md047;

use crate::Rule;

/// The registry of rules this card runs, in the order their findings are merged (ADR-0006 then
/// sorts all of one file's findings by `(line, column)`, so registry order only matters as the
/// tie-break for findings at the same location).
pub const RULES: &[&dyn Rule] = &[&Md009, &Md047];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    #[test]
    fn registry_lists_both_rules_in_scope() {
        assert_eq!(RULES.len(), 2);
        let doc = Document::from_source("abc  ");
        // Exercise every registered rule through the registry itself, not just its own module.
        let total: usize = RULES.iter().map(|rule| rule.check(&doc).len()).sum();
        assert_eq!(total, 2); // trailing whitespace (MD009) + missing trailing newline (MD047)
    }
}
