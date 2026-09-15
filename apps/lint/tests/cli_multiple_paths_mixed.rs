//! Multiple paths mixing a clean file, a dirty file, and a directory report findings from
//! every path, in input order, and exit 1 if any produced a finding.

mod support;

#[test]
fn mixed_paths_report_from_every_path_in_input_order_and_exit_one() {
    let dir = support::temp_dir("multiple-paths-mixed");
    let clean = support::write_file(&dir, "clean.md", "clean\n");
    let dirty = support::write_file(&dir, "dirty.md", "trail  \n");
    let sub = dir.join("sub");
    let sub_dirty = support::write_file(&dir, "sub/also_dirty.md", "trail  \n");

    let output = support::run_lint(&[
        clean.to_str().unwrap(),
        dirty.to_str().unwrap(),
        sub.to_str().unwrap(),
    ]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!(
        "{}:1:6 MD009 trailing whitespace\n{}:1:6 MD009 trailing whitespace\n",
        dirty.display(),
        sub_dirty.display()
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn an_all_clean_mix_exits_zero() {
    let dir = support::temp_dir("multiple-paths-all-clean");
    let clean_a = support::write_file(&dir, "a.md", "clean\n");
    let sub = dir.join("sub");
    support::write_file(&dir, "sub/b.md", "clean\n");

    let output = support::run_lint(&[clean_a.to_str().unwrap(), sub.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn an_all_dirty_mix_reports_all_findings_and_exits_one() {
    let dir = support::temp_dir("multiple-paths-all-dirty");
    let a = support::write_file(&dir, "a.md", "trail  \n");
    let b = support::write_file(&dir, "b.md", "trail  \n");

    let output = support::run_lint(&[a.to_str().unwrap(), b.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!(
        "{}:1:6 MD009 trailing whitespace\n{}:1:6 MD009 trailing whitespace\n",
        a.display(),
        b.display()
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
