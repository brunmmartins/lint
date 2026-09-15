//! A directory tree containing a symbolic-link loop never causes `lint` to hang (or panic).
//! Unix-only, because it builds the loop with a Unix symlink. It asserts process termination within
//! a bounded wall-clock timeout — non-termination is the failure mode under test.

mod support;

#[cfg(unix)]
#[test]
fn a_symlink_loop_does_not_hang_lint() {
    use std::os::unix::fs::symlink;
    use std::time::Duration;

    let dir = support::temp_dir("symlink-loop");
    support::write_file(&dir, "real.md", "trail  \n");
    // `dir/loop` points back at `dir` itself: a directory tree with a symlink loop.
    symlink(&dir, dir.join("loop")).unwrap();

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_lint"))
        .arg(dir.to_str().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn the built lint binary");

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().expect("failed to poll child process") {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            std::fs::remove_dir_all(&dir).unwrap();
            panic!("lint did not terminate within 10 seconds on a symlink loop");
        }
        std::thread::sleep(Duration::from_millis(20));
    };

    assert_eq!(status.code(), Some(1)); // real.md's trailing-space finding, no fault

    std::fs::remove_dir_all(&dir).unwrap();
}
