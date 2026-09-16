//! MD001 (`heading-increment`): a heading more than one level deeper than the heading before it is
//! reported, whatever style it is written in and whatever it is nested in.

mod support;

fn skipped(expected: u8, found: u8) -> String {
    format!("MD001 heading level skipped (expected h{expected}, found h{found})")
}

#[test]
fn skipped_levels_are_reported() {
    let dir = support::temp_dir("md001-skipped");
    let source = "# One\n\n### Three\n\n#### Four\n\n###### Six\n\n## Two\n\n#### Four again\n";
    let file = support::write_file(&dir, "levels.md", source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:3:1 {}\n{path}:7:1 {}\n{path}:11:1 {}\n",
        skipped(2, 3),
        skipped(5, 6),
        skipped(3, 4)
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn first_heading_may_have_any_level() {
    let dir = support::temp_dir("md001-first");
    let file = support::write_file(
        &dir,
        "first.md",
        "### Start\n\n#### Next\n\n## Up\n\n### Down\n",
    );

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn setext_levels_count() {
    let dir = support::temp_dir("md001-setext");
    let file = support::write_file(&dir, "setext.md", "Title\n=====\n\nSub\n---\n\n#### Deep\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!("{}:7:1 {}\n", file.display(), skipped(3, 4));
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn nested_headings_count() {
    let dir = support::temp_dir("md001-nested");
    let file = support::write_file(
        &dir,
        "nested.md",
        "# A\n\n- item\n\n  ### Nested\n\n> #### Quoted\n",
    );

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!("{}:5:1 {}\n", file.display(), skipped(2, 3));
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
