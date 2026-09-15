//! MD013 (`line-length`): a line longer than 80 characters is reported at column 81, unless it has
//! no whitespace from column 81 onward.

mod support;

fn too_long(found: usize) -> String {
    format!("MD013 line too long (expected at most 80 characters, found {found})")
}

#[test]
fn reports_long_lines_at_column_81() {
    let dir = support::temp_dir("md013-limit");
    let exactly_eighty = "word ".repeat(15) + "abcde";
    assert_eq!(exactly_eighty.chars().count(), 80);
    let source = format!(
        "{exactly_eighty}\n{}\n{}\n",
        "a".repeat(89) + " bbbbb",
        "a".repeat(80) + " b",
    );
    let file = support::write_file(&dir, "long.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:2:81 {}\n{path}:3:81 {}\n",
        too_long(95),
        too_long(82)
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unbreakable_overflow_is_exempt() {
    let dir = support::temp_dir("md013-exempt");
    let source = format!(
        "{}\n{}\n{}\n",
        "x".repeat(120),
        "a".repeat(79) + " https://example.com/" + &"x".repeat(30),
        "a ".repeat(30) + &"b".repeat(70),
    );
    let file = support::write_file(&dir, "exempt.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn headings_code_and_tables_share_the_limit() {
    let dir = support::temp_dir("md013-kinds");
    let heading = "# ".to_string() + &"a".repeat(79) + " " + &"b".repeat(8);
    let code = "a".repeat(80) + " " + &"b".repeat(9);
    let row = "| ".to_string() + &"a".repeat(78) + " | " + &"b".repeat(5) + " |";
    for line in [&heading, &code, &row] {
        assert_eq!(line.chars().count(), 90);
    }
    let source = [
        heading.as_str(),
        "",
        "```text",
        code.as_str(),
        "```",
        "",
        "| h | i |",
        "| --- | --- |",
        row.as_str(),
    ]
    .join("\n")
        + "\n";
    let file = support::write_file(&dir, "kinds.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let message = too_long(90);
    let expected = format!("{path}:1:81 {message}\n{path}:4:81 {message}\n{path}:9:81 {message}\n");
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn counts_characters_not_bytes() {
    let dir = support::temp_dir("md013-characters");
    let eighty = "é".repeat(40) + " " + &"é".repeat(39);
    assert_eq!(eighty.chars().count(), 80);
    assert_eq!(eighty.len(), 159);
    let source = format!(
        "{eighty}\n{}\n{}\n{}\n",
        "é".repeat(81) + " é",
        "é\tx",
        "\t".to_string() + &"a".repeat(79),
    );
    let file = support::write_file(&dir, "characters.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:2:81 {}\n{path}:3:2 MD010 hard tab\n{path}:4:1 MD010 hard tab\n",
        too_long(83)
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
