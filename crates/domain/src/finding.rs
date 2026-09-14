use crate::{Location, RuleId};

/// One style problem a [`Rule`](crate::Rule) reported at a specific location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    rule: RuleId,
    location: Location,
    message: String,
}

impl Finding {
    /// Builds a `Finding`. Infallible: every field is already a validated type.
    #[must_use]
    pub fn new(rule: RuleId, location: Location, message: impl Into<String>) -> Self {
        Self {
            rule,
            location,
            message: message.into(),
        }
    }

    /// The rule that produced this finding.
    #[must_use]
    pub fn rule(&self) -> RuleId {
        self.rule
    }

    /// Where in the document the finding applies.
    #[must_use]
    pub fn location(&self) -> Location {
        self.location
    }

    /// The human-readable message, with no file contents echoed into it (brief §8).
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_constructed_values() {
        let location = Location::new(1, 1).unwrap();
        let finding = Finding::new(RuleId::Md009, location, "trailing whitespace");
        assert_eq!(finding.rule(), RuleId::Md009);
        assert_eq!(finding.location(), location);
        assert_eq!(finding.message(), "trailing whitespace");
    }
}
