//! When the reader of `lint`'s standard output goes away early, as `head -1` does, `lint` stops
//! without checking the remaining paths and exits 2, with nothing on stderr: no panic text, no
//! message about the pipe, and no fault from a path it never reached. Unix-only, because it relies
//! on how a pipe without a reader fails.

mod support;

#[cfg(unix)]
#[test]
fn closing_stdout_after_one_line_stops_with_exit_two_and_empty_stderr() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let dir = support::temp_dir("closed-stdout");
    // Far more output than a pipe buffer holds, so lint is still writing when the reader leaves.
    let many = support::write_file(&dir, "many.md", &" \n".repeat(100_000));
    // Would be reported on stderr if lint went on to read it.
    let bad = dir.join("bad.md");
    std::fs::write(&bad, [0xFF]).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_lint"))
        .arg(&many)
        .arg(&bad)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn the built lint binary");
    let stderr = support::drain(child.stderr.take().expect("stderr was not piped"));

    let mut stdout = BufReader::new(child.stdout.take().expect("stdout was not piped"));
    let mut first_line = String::new();
    stdout.read_line(&mut first_line).unwrap();
    drop(stdout);

    let status = support::wait_with_deadline(&mut child, Duration::from_secs(10));
    let stderr = String::from_utf8(stderr.join().expect("stderr reader thread panicked")).unwrap();

    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        first_line.starts_with(&format!("{}:1:1 MD009 ", many.display())),
        "unexpected first line: {first_line:?}"
    );
    assert_eq!(stderr, "");
    assert_eq!(status.code(), Some(2));
}
