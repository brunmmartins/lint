//! The rule implementations, and the static registry the engine iterates.

mod md009;
mod md010;
mod md012;
mod md013;
mod md047;

pub use md009::Md009;
pub use md010::Md010;
pub use md012::Md012;
pub use md013::Md013;
pub use md047::Md047;

use crate::Rule;

/// The registry of rules `lint` runs, in ascending rule-ID order.
///
/// Callers sort one file's findings by location and then by rule ID, so the order here does not
/// decide output order.
pub const RULES: &[&dyn Rule] = &[&Md009, &Md010, &Md012, &Md013, &Md047];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, RuleId};

    #[test]
    fn registry_runs_every_rule_in_scope() {
        assert_eq!(RULES.len(), 5);
        // One source that each rule reports on once: a trailing tab (MD009, MD010), a second
        // consecutive blank line (MD012), an over-long wrappable line (MD013), and a missing final
        // newline (MD047).
        let source = "abc\t\n\n\n".to_string() + &"a ".repeat(41) + "b";
        let doc = Document::from_source(&source);

        // Exercise every registered rule through the registry itself, not just its own module.
        let mut rules: Vec<RuleId> = RULES
            .iter()
            .flat_map(|rule| rule.check(&doc))
            .map(|finding| finding.rule())
            .collect();
        rules.sort();

        assert_eq!(
            rules,
            [
                RuleId::Md009,
                RuleId::Md010,
                RuleId::Md012,
                RuleId::Md013,
                RuleId::Md047
            ]
        );
    }
}
