//! Writing results as each file is checked keeps each stream's order and text: stdout holds the
//! findings in input order, then walk order, then per-file order; stderr holds the faults in the
//! same processing order; and a fault still makes the exit status 2.

mod support;

#[test]
fn stdout_and_stderr_keep_input_walk_and_per_file_order() {
    let dir = support::temp_dir("stream-order");
    let dirty_a = support::write_file(&dir, "dirty-a.md", "trail  \n\tx\n");
    let missing = dir.join("missing.md");
    let sub = dir.join("dir");
    let b = support::write_file(&dir, "dir/b.md", "trail  \n");
    let c = sub.join("c.md");
    std::fs::write(&c, [0xFF]).unwrap();
    let d = support::write_file(&dir, "dir/d.md", "a\tb\n");

    let output = support::run_lint(&[
        dirty_a.to_str().unwrap(),
        missing.to_str().unwrap(),
        sub.to_str().unwrap(),
    ]);

    std::fs::remove_dir_all(&dir).unwrap();
    let expected_stdout = format!(
        "{a}:1:6 MD009 trailing whitespace\n\
         {a}:2:1 MD010 hard tab\n\
         {b}:1:6 MD009 trailing whitespace\n\
         {d}:1:2 MD010 hard tab\n",
        a = dirty_a.display(),
        b = b.display(),
        d = d.display(),
    );
    let expected_stderr = format!(
        "{}: no such file or directory\n{}: not valid UTF-8\n",
        missing.display(),
        c.display()
    );
    assert_eq!(support::stdout(&output), expected_stdout);
    assert_eq!(support::stderr(&output), expected_stderr);
    assert_eq!(output.status.code(), Some(2));
}
