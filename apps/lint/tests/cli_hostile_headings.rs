//! Hostile heading input neither panics nor hangs, and still reports exactly the findings the
//! rules define. Each run has a generous deadline: a hang guard, not a performance budget.

mod support;

use std::time::Duration;

const DEADLINE: Duration = Duration::from_secs(30);

const MD018: &str = "MD018 no space after hash on atx style heading";
const ABOVE: &str = "MD022 missing blank line above heading";
const BELOW: &str = "MD022 missing blank line below heading";
const MD025: &str = "MD025 multiple top-level headings in the same document";

fn too_long(found: usize) -> String {
    format!("MD013 line too long (expected at most 80 characters, found {found})")
}

/// Runs `lint` on a file holding `source`, and returns its exit code and stdout.
fn run_on(label: &str, source: &str) -> (Option<i32>, String, std::path::PathBuf) {
    let dir = support::temp_dir(label);
    let file = support::write_file(&dir, "hostile.md", source);
    let path = file.to_str().expect("the temp path is not UTF-8");
    let output = support::run_lint_with_deadline(&[path], DEADLINE);
    assert_eq!(support::stderr(&output), "", "case {label}");
    let stdout = support::stdout(&output);
    std::fs::remove_dir_all(&dir).expect("failed to remove the temp dir");
    (output.status.code(), stdout, file)
}

/// Compares a large stdout without printing megabytes on failure.
fn assert_stdout(actual: &str, expected: &str, label: &str) {
    assert_eq!(
        actual.lines().count(),
        expected.lines().count(),
        "case {label}: line counts differ"
    );
    assert!(actual == expected, "case {label}: stdout differs");
}

#[test]
fn hundred_thousand_hash_lines() {
    let (code, stdout, file) = run_on("hostile-hash-lines", &"#x\n".repeat(100_000));

    assert_eq!(code, Some(1));
    let path = file.display();
    let expected: String = (1..=100_000)
        .map(|line| format!("{path}:{line}:1 {MD018}\n"))
        .collect();
    assert_stdout(&stdout, &expected, "hundred thousand hash lines");
}

#[test]
fn hundred_thousand_adjacent_headings() {
    let (code, stdout, file) = run_on("hostile-adjacent-headings", &"# h\n".repeat(100_000));

    assert_eq!(code, Some(1));
    let path = file.display();
    let mut expected = String::new();
    for line in 1..=100_000_usize {
        if line > 1 {
            expected.push_str(&format!("{path}:{line}:1 {ABOVE}\n"));
        }
        if line < 100_000 {
            expected.push_str(&format!("{path}:{line}:1 {BELOW}\n"));
        }
        if line > 1 {
            expected.push_str(&format!("{path}:{line}:1 {MD025}\n"));
        }
    }
    assert_stdout(&stdout, &expected, "hundred thousand adjacent headings");
    // 199,998 MD022 findings and 99,999 MD025 findings.
    assert_eq!(stdout.lines().count(), 299_997);
}

#[test]
fn deeply_nested_block_quote_heading() {
    let (code, stdout, file) = run_on("hostile-nested-quote", &("> ".repeat(10_000) + "# H\n"));

    assert_eq!(code, Some(1));
    assert_eq!(
        stdout,
        format!("{}:1:81 {}\n", file.display(), too_long(20_003))
    );
}

#[test]
fn deeply_nested_list_heading() {
    let (code, stdout, file) = run_on("hostile-nested-list", &("- ".repeat(10_000) + "# H\n"));

    assert_eq!(code, Some(1));
    assert_eq!(
        stdout,
        format!("{}:1:81 {}\n", file.display(), too_long(20_003))
    );
}

#[test]
fn a_million_hashes_then_text() {
    let (code, stdout, file) = run_on("hostile-million-hashes", &("#".repeat(999_999) + "x\n"));

    assert_eq!(code, Some(1));
    assert_eq!(stdout, format!("{}:1:1 {MD018}\n", file.display()));
}

#[test]
fn a_million_character_comment_line_above_a_heading() {
    let source = "<!-- -->".repeat(125_000) + "\n# H\n";
    let (code, stdout, file) = run_on("hostile-comment-line", &source);

    assert_eq!(code, Some(1));
    assert_eq!(
        stdout,
        format!("{}:1:81 {}\n", file.display(), too_long(1_000_000))
    );
}

#[test]
fn many_comment_blocks_before_the_title() {
    let source = "<!-- c -->\n".repeat(100_000) + "# T\n\n# U\n";
    let (code, stdout, file) = run_on("hostile-comment-blocks", &source);

    assert_eq!(code, Some(1));
    assert_eq!(stdout, format!("{}:100003:1 {MD025}\n", file.display()));
}

#[test]
fn many_html_blocks_hide_hash_lines() {
    let mut source = "<div>\n#x\n</div>\n\n".repeat(25_000);
    source.pop();
    let (code, stdout, _) = run_on("hostile-html-blocks", &source);

    assert_eq!(code, Some(0));
    assert_eq!(stdout, "");
}
