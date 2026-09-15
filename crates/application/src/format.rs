use std::path::Path;

use lint_domain::Finding;

/// Formats one finding as the fixed `path:line:col RULE message` text line.
#[must_use]
pub fn format_finding(path: &Path, finding: &Finding) -> String {
    format!(
        "{}:{} {} {}",
        path.display(),
        finding.location(),
        finding.rule(),
        finding.message()
    )
}

/// Decides the process exit code from whether any fault or any finding occurred, with a fixed
/// precedence: a fault (I/O-class problem) always wins over a finding, regardless of how
/// many of either occurred, because it means the invocation could not be trusted to have
/// inspected everything asked of it.
///
/// Returns `2` when `has_fault` is `true`, `1` when `has_fault` is `false` and `has_finding` is
/// `true`, `0` otherwise.
#[must_use]
pub fn decide_exit_code(has_fault: bool, has_finding: bool) -> u8 {
    if has_fault {
        2
    } else if has_finding {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lint_domain::{FindingMessage, Location, RuleId};
    use std::path::PathBuf;

    #[test]
    fn format_finding_matches_the_fixed_shape() {
        let finding = Finding::new(
            RuleId::Md009,
            Location::new(3, 5).unwrap(),
            FindingMessage::TrailingWhitespace,
        );
        let line = format_finding(&PathBuf::from("doc.md"), &finding);
        assert_eq!(line, "doc.md:3:5 MD009 trailing whitespace");
    }

    #[test]
    fn no_fault_no_finding_exits_zero() {
        assert_eq!(decide_exit_code(false, false), 0);
    }

    #[test]
    fn finding_without_fault_exits_one() {
        assert_eq!(decide_exit_code(false, true), 1);
    }

    #[test]
    fn fault_without_finding_exits_two() {
        assert_eq!(decide_exit_code(true, false), 2);
    }

    #[test]
    fn fault_takes_precedence_over_a_finding_present_in_the_same_run() {
        // The mixed case: a fault still wins when a finding is also present in the same run.
        assert_eq!(decide_exit_code(true, true), 2);
    }
}
