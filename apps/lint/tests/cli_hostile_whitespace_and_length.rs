//! Hostile whitespace and line-length input neither panics nor hangs, and still reports exactly
//! the findings the rules define. Each run has a generous deadline: a hang guard, not a
//! performance budget.

mod support;

use std::time::Duration;

const DEADLINE: Duration = Duration::from_secs(30);

fn blank_lines_message(found: usize) -> String {
    format!("MD012 multiple consecutive blank lines (expected at most 1, found {found})")
}

#[test]
fn million_character_line() {
    let dir = support::temp_dir("hostile-long-line");
    let line = "a ".repeat(499_999) + "ab";
    assert_eq!(line.chars().count(), 1_000_000);
    let file = support::write_file(&dir, "long.md", &(line + "\n"));

    let output = support::run_lint_with_deadline(&[file.to_str().unwrap()], DEADLINE);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!(
        "{}:1:81 MD013 line too long (expected at most 80 characters, found 1000000)\n",
        file.display()
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn hundred_thousand_tab_runs() {
    let dir = support::temp_dir("hostile-tabs");
    let file = support::write_file(&dir, "tabs.md", &("\ta".repeat(100_000) + "\n"));

    let output = support::run_lint_with_deadline(&[file.to_str().unwrap()], DEADLINE);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let mut expected = String::new();
    for run in 0..100_000 {
        let column = 2 * run + 1;
        expected.push_str(&format!("{path}:1:{column} MD010 hard tab\n"));
        if column == 81 {
            expected.push_str(&format!(
                "{path}:1:81 MD013 line too long (expected at most 80 characters, found 200000)\n"
            ));
        }
    }
    let stdout = support::stdout(&output);
    assert_eq!(stdout.lines().count(), 100_001);
    assert!(
        stdout == expected,
        "stdout differs from the expected findings"
    );
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn hundred_thousand_blank_lines() {
    let dir = support::temp_dir("hostile-blank-lines");
    let source = "top\n".to_string() + &"\n".repeat(100_000) + "bottom\n";
    let file = support::write_file(&dir, "blank.md", &source);

    let output = support::run_lint_with_deadline(&[file.to_str().unwrap()], DEADLINE);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected: String = (3..=100_001)
        .map(|line| format!("{path}:{line}:1 {}\n", blank_lines_message(line - 1)))
        .collect();
    let stdout = support::stdout(&output);
    assert_eq!(stdout.lines().count(), 99_999);
    assert!(
        stdout == expected,
        "stdout differs from the expected findings"
    );
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unclosed_fence_before_hundred_thousand_blank_lines() {
    let dir = support::temp_dir("hostile-unclosed-fence");
    let source = "```\n".to_string() + &"\n".repeat(100_000) + "end\n";
    let file = support::write_file(&dir, "unclosed.md", &source);

    let output = support::run_lint_with_deadline(&[file.to_str().unwrap()], DEADLINE);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn many_code_blocks_do_not_hang() {
    let dir = support::temp_dir("hostile-code-blocks");
    let source = "```\n\n\n```\n".repeat(100_000) + "end\n";
    let file = support::write_file(&dir, "blocks.md", &source);

    let output = support::run_lint_with_deadline(&[file.to_str().unwrap()], DEADLINE);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
