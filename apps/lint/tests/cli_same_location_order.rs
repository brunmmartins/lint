//! Findings from different rules at the same line and column of one file print in ascending
//! rule-ID order.

mod support;

#[test]
fn same_location_findings_print_in_rule_id_order() {
    let dir = support::temp_dir("same-location-order");
    let cases = [
        (
            "tab.md",
            "abc\t\n",
            vec!["1:4 MD009 trailing whitespace", "1:4 MD010 hard tab"],
        ),
        (
            "blank.md",
            "one\n\n  \ntwo\n",
            vec![
                "3:1 MD009 trailing whitespace",
                "3:1 MD012 multiple consecutive blank lines (expected at most 1, found 2)",
            ],
        ),
        (
            "end.md",
            "abc\n\n\n",
            vec![
                "3:1 MD012 multiple consecutive blank lines (expected at most 1, found 2)",
                "3:1 MD047 more than one trailing newline",
            ],
        ),
    ];

    for (name, source, lines) in cases {
        let file = support::write_file(&dir, name, source);

        let output = support::run_lint(&[file.to_str().unwrap()]);

        assert_eq!(output.status.code(), Some(1), "case {name}");
        let expected: String = lines
            .iter()
            .map(|line| format!("{}:{line}\n", file.display()))
            .collect();
        assert_eq!(support::stdout(&output), expected, "case {name}");
    }

    std::fs::remove_dir_all(&dir).unwrap();
}
