use std::fmt;

/// The closed set of rules `lint` currently implements.
///
/// `Display` prints the rule's canonical identifier (`"MD009"`, `"MD047"`), matching the
/// `path:line:col RULE message` output format. Variants are declared in ascending identifier order,
/// so the derived `Ord` orders rules by identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RuleId {
    /// `heading-increment`: a heading is more than one level deeper than the heading before it.
    Md001,
    /// `no-trailing-spaces`: a line ends with trailing whitespace.
    Md009,
    /// `no-hard-tabs`: a line contains a hard tab character.
    Md010,
    /// `no-multiple-blanks`: more than one consecutive blank line outside code blocks.
    Md012,
    /// `line-length`: a line is longer than the limit and could be wrapped.
    Md013,
    /// `no-missing-space-atx`: a line opens with `#` characters that no space follows.
    Md018,
    /// `blanks-around-headings`: a heading has no blank line directly above or below it.
    Md022,
    /// `single-h1`: a document with a top-level title has another top-level heading.
    Md025,
    /// `single-trailing-newline`: the file does not end with exactly one trailing newline.
    Md047,
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match self {
            RuleId::Md001 => "MD001",
            RuleId::Md009 => "MD009",
            RuleId::Md010 => "MD010",
            RuleId::Md012 => "MD012",
            RuleId::Md013 => "MD013",
            RuleId::Md018 => "MD018",
            RuleId::Md022 => "MD022",
            RuleId::Md025 => "MD025",
            RuleId::Md047 => "MD047",
        };
        f.write_str(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [RuleId; 9] = [
        RuleId::Md001,
        RuleId::Md009,
        RuleId::Md010,
        RuleId::Md012,
        RuleId::Md013,
        RuleId::Md018,
        RuleId::Md022,
        RuleId::Md025,
        RuleId::Md047,
    ];

    #[test]
    fn display_prints_canonical_rule_ids() {
        let printed: Vec<String> = ALL.iter().map(ToString::to_string).collect();
        assert_eq!(
            printed,
            [
                "MD001", "MD009", "MD010", "MD012", "MD013", "MD018", "MD022", "MD025", "MD047"
            ]
        );
    }

    #[test]
    fn order_follows_ascending_rule_ids() {
        let mut by_ord = ALL;
        by_ord.reverse();
        by_ord.sort();

        let mut by_id = ALL;
        by_id.reverse();
        by_id.sort_by_key(ToString::to_string);

        assert_eq!(by_ord, ALL);
        assert_eq!(by_ord, by_id);
    }
}
