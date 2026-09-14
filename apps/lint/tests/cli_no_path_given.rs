//! AC6: no path argument prints a usage message to stderr and exits 2.

mod support;

#[test]
fn no_path_argument_prints_usage_to_stderr_and_exits_two() {
    let output = support::run_lint(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(support::stdout(&output), "");
    assert!(!support::stderr(&output).is_empty());
}
