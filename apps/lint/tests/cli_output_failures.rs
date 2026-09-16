//! Standard output that cannot be written never makes `lint` panic. A full device exits 2 with one
//! stderr line that names standard output and the reason; stdout closed before start behaves as
//! before; and a standard error with no reader does not panic either.

mod support;

#[cfg(target_os = "linux")]
#[test]
fn stdout_on_dev_full_exits_two_with_one_stderr_line() {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let dir = support::temp_dir("output-dev-full");
    let dirty = support::write_file(&dir, "dirty.md", "trail  \n");
    let full = std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_lint"))
        .arg(&dirty)
        .stdout(Stdio::from(full))
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn the built lint binary");
    let stderr = support::drain(child.stderr.take().expect("stderr was not piped"));
    let status = support::wait_with_deadline(&mut child, Duration::from_secs(10));
    let stderr = String::from_utf8(stderr.join().expect("stderr reader thread panicked")).unwrap();

    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(stderr.lines().count(), 1, "stderr: {stderr:?}");
    assert!(
        stderr.starts_with("standard output: cannot be written ("),
        "stderr: {stderr:?}"
    );
    assert!(stderr.contains("os error 28"), "stderr: {stderr:?}");
    assert!(stderr.ends_with(")\n"), "stderr: {stderr:?}");
    assert_eq!(status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn stdout_closed_before_start_exits_one() {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let dir = support::temp_dir("output-closed-before-start");
    let dirty = support::write_file(&dir, "dirty.md", "trail  \n");

    // The shell closes descriptor 1 before running lint; std has no safe way to do that itself.
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(r#"exec "$0" "$1" >&-"#)
        .arg(env!("CARGO_BIN_EXE_lint"))
        .arg(&dirty)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn sh");
    let stderr = support::drain(child.stderr.take().expect("stderr was not piped"));
    let status = support::wait_with_deadline(&mut child, Duration::from_secs(10));
    let stderr = String::from_utf8(stderr.join().expect("stderr reader thread panicked")).unwrap();

    std::fs::remove_dir_all(&dir).unwrap();
    assert!(!stderr.contains("panicked"), "stderr: {stderr:?}");
    assert_eq!(status.code(), Some(1));
}

#[cfg(unix)]
#[test]
fn a_stderr_pipe_without_a_reader_does_not_panic() {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let dir = support::temp_dir("output-broken-stderr");
    let missing_a = dir.join("missing-a.md");
    let missing_b = dir.join("missing-b.md");
    let (reader, writer) = std::io::pipe().unwrap();
    drop(reader);

    let mut child = Command::new(env!("CARGO_BIN_EXE_lint"))
        .arg(&missing_a)
        .arg(&missing_b)
        .stdout(Stdio::piped())
        .stderr(Stdio::from(writer))
        .spawn()
        .expect("failed to spawn the built lint binary");
    let stdout = support::drain(child.stdout.take().expect("stdout was not piped"));
    let status = support::wait_with_deadline(&mut child, Duration::from_secs(10));
    let stdout = stdout.join().expect("stdout reader thread panicked");

    std::fs::remove_dir_all(&dir).unwrap();
    assert!(stdout.is_empty());
    assert_eq!(status.code(), Some(2));
}
