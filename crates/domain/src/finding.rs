use crate::{FindingMessage, Location, RuleId};

/// One style problem a [`Rule`](crate::Rule) reported at a specific location.
///
/// A finding holds only fixed-size data, so it is `Copy` and a file with very many findings costs
/// a fixed amount of memory per finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    rule: RuleId,
    location: Location,
    message: FindingMessage,
}

impl Finding {
    /// Builds a `Finding`. Infallible: every field is already a validated type.
    #[must_use]
    pub fn new(rule: RuleId, location: Location, message: FindingMessage) -> Self {
        Self {
            rule,
            location,
            message,
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

    /// The message, which never echoes file contents.
    #[must_use]
    pub fn message(&self) -> FindingMessage {
        self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_constructed_values() {
        let location = Location::new(1, 1).unwrap();
        let finding = Finding::new(RuleId::Md009, location, FindingMessage::TrailingWhitespace);
        assert_eq!(finding.rule(), RuleId::Md009);
        assert_eq!(finding.location(), location);
        assert_eq!(finding.message().to_string(), "trailing whitespace");
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn finding_stays_fixed_size() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<Finding>();
        assert!(size_of::<Finding>() <= 48, "{} bytes", size_of::<Finding>());
    }
}
