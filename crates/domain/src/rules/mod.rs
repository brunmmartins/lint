//! The rule implementations, and the static registry the engine iterates.

mod line_text;
mod md001;
mod md009;
mod md010;
mod md012;
mod md013;
mod md018;
mod md022;
mod md025;
mod md047;

pub use md001::Md001;
pub use md009::Md009;
pub use md010::Md010;
pub use md012::Md012;
pub use md013::Md013;
pub use md018::Md018;
pub use md022::Md022;
pub use md025::Md025;
pub use md047::Md047;

use crate::Rule;

/// The registry of rules `lint` runs, in ascending rule-ID order.
///
/// Callers sort one file's findings by location and then by rule ID, so the order here does not
/// decide output order.
pub const RULES: &[&dyn Rule] = &[
    &Md001, &Md009, &Md010, &Md012, &Md013, &Md018, &Md022, &Md025, &Md047,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockKind, Document, HeadingLevel, RuleId};

    #[test]
    fn registry_runs_every_rule_in_scope() {
        assert_eq!(RULES.len(), 9);
        // One source that each rule reports on exactly once.
        //
        // Lines: 1 a title, 2 blank, 3 a level-3 heading whose level skips one (MD001) and that
        // has no blank line below it (MD022), 4 a line opening with hashes and no space (MD018),
        // 5 blank, 6 blank (MD012), 7 a second level-1 heading (MD025) ending in a tab (MD009,
        // MD010) and long enough to wrap (MD013), with no final newline (MD047).
        let source = "# t\n\n### s\n#x\n\n\n# ".to_string() + &"a ".repeat(40) + "b\t";
        let blocks = [
            (BlockKind::Heading(HeadingLevel::H1), 0..3),
            (BlockKind::Heading(HeadingLevel::H3), 5..10),
            (BlockKind::Heading(HeadingLevel::H1), 16..source.len()),
        ];
        let doc = Document::from_source_and_blocks(&source, blocks);

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
                RuleId::Md001,
                RuleId::Md009,
                RuleId::Md010,
                RuleId::Md012,
                RuleId::Md013,
                RuleId::Md018,
                RuleId::Md022,
                RuleId::Md025,
                RuleId::Md047
            ]
        );
    }
}
