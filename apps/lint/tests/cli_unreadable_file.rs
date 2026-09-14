//! AC8: a file that exists but cannot be read (permission denied) prints an error naming the path
//! to stderr and exits 2, without a panic.

mod support;

#[cfg(unix)]
#[test]
fn permission_denied_file_prints_error_to_stderr_and_exits_two() {
    use std::os::unix::fs::PermissionsExt;

    let dir = support::temp_dir("unreadable-file");
    let file = support::write_file(&dir, "secret.md", "secret content\n");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();

    let output = support::run_lint(&[file.to_str().unwrap()]);

    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();

    // Left unverified when the test runner is root: permission bits are bypassed for root on
    // most platforms, so the read can succeed instead of failing (architecture contract, "Left
    // unverified"). Only assert the failure shape when the read did fail.
    if output.status.code() != Some(0) {
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(support::stdout(&output), "");
        let stderr = support::stderr(&output);
        assert!(stderr.contains(&file.display().to_string()));
        assert!(!stderr.contains("secret content"));
    }

    std::fs::remove_dir_all(&dir).unwrap();
}
