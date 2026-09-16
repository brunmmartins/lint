//! Findings from the heading rules sort with every other rule's by location and then rule ID, and
//! two findings one rule reports at one location keep the order it reported them in.

mod support;

#[test]
fn same_location_findings_sort_by_rule_then_report_order() {
    let dir = support::temp_dir("heading-order-same-location");
    let file = support::write_file(&dir, "order.md", "# A\n### B\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:1:1 MD022 missing blank line below heading\n\
         {path}:2:1 MD001 heading level skipped (expected h2, found h3)\n\
         {path}:2:1 MD022 missing blank line above heading\n"
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_heading_rule_finding_sorts_before_a_later_column() {
    let dir = support::temp_dir("heading-order-columns");
    let file = support::write_file(&dir, "columns.md", "#\t\n#\t\n#\t\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:1:1 MD022 missing blank line below heading\n\
         {path}:1:2 MD009 trailing whitespace\n\
         {path}:1:2 MD010 hard tab\n\
         {path}:2:1 MD022 missing blank line above heading\n\
         {path}:2:1 MD022 missing blank line below heading\n\
         {path}:2:1 MD025 multiple top-level headings in the same document\n\
         {path}:2:2 MD009 trailing whitespace\n\
         {path}:2:2 MD010 hard tab\n\
         {path}:3:1 MD022 missing blank line above heading\n\
         {path}:3:1 MD025 multiple top-level headings in the same document\n\
         {path}:3:2 MD009 trailing whitespace\n\
         {path}:3:2 MD010 hard tab\n"
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn crlf_does_not_change_heading_detection() {
    let dir = support::temp_dir("heading-order-crlf");
    let file = support::write_file(&dir, "crlf.md", "# A\r\ntext\r\n\r\n### C\r\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:1:1 MD022 missing blank line below heading\n\
         {path}:4:1 MD001 heading level skipped (expected h2, found h3)\n"
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
