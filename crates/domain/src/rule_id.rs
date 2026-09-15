use std::fmt;

/// The closed set of rules `lint` currently implements.
///
/// `Display` prints the rule's canonical identifier (`"MD009"`, `"MD047"`), matching the
/// `path:line:col RULE message` output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RuleId {
    /// `no-trailing-spaces`: a line ends with trailing whitespace.
    Md009,
    /// `single-trailing-newline`: the file does not end with exactly one trailing newline.
    Md047,
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleId::Md009 => write!(f, "MD009"),
            RuleId::Md047 => write!(f, "MD047"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_prints_canonical_rule_ids() {
        assert_eq!(RuleId::Md009.to_string(), "MD009");
        assert_eq!(RuleId::Md047.to_string(), "MD047");
    }
}
