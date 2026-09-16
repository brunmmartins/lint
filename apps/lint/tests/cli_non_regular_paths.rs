//! An explicitly named path that is not a regular file, or a symlink to one, is rejected without
//! blocking on it or reading from it: stdout stays empty, one stderr line names the path, and the
//! exit status is 2. Other paths in the same run are still checked, and a walked directory still
//! skips such entries. Unix-only, because the fixtures are named pipes and Unix symlinks.

mod support;

#[cfg(unix)]
mod unix {
    use std::path::Path;
    use std::time::Duration;

    use super::support;

    const DEADLINE: Duration = Duration::from_secs(10);

    fn make_fifo(path: &Path) {
        let status = std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .expect("failed to run mkfifo");
        assert!(status.success(), "mkfifo failed for {}", path.display());
    }

    fn assert_rejected_alone(path: &Path) {
        let output = support::run_lint_with_deadline(
            &[path.to_str().expect("temp paths are UTF-8")],
            DEADLINE,
        );

        assert_eq!(support::stdout(&output), "");
        assert_eq!(
            support::stderr(&output),
            format!("{}: not a regular file\n", path.display())
        );
        assert_eq!(output.status.code(), Some(2));
    }

    #[test]
    fn a_fifo_without_a_writer_is_rejected_without_blocking() {
        let dir = support::temp_dir("non-regular-fifo");
        let fifo = dir.join("pipe.md");
        make_fifo(&fifo);

        assert_rejected_alone(&fifo);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_symlink_to_dev_zero_is_rejected_without_reading() {
        let dir = support::temp_dir("non-regular-dev-zero");
        let evil = dir.join("evil.md");
        std::os::unix::fs::symlink("/dev/zero", &evil).unwrap();

        assert_rejected_alone(&evil);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_symlink_to_a_directory_is_not_a_regular_file() {
        let dir = support::temp_dir("non-regular-dir-link");
        let target = dir.join("target");
        std::fs::create_dir(&target).unwrap();
        let link = dir.join("dlink.md");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert_rejected_alone(&link);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_rejected_path_does_not_hide_findings_in_the_next_path() {
        let dir = support::temp_dir("non-regular-then-dirty");
        let fifo = dir.join("pipe.md");
        make_fifo(&fifo);
        let dirty = support::write_file(&dir, "dirty.md", "trail  \n");

        let output = support::run_lint_with_deadline(
            &[fifo.to_str().unwrap(), dirty.to_str().unwrap()],
            DEADLINE,
        );

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            support::stdout(&output),
            format!("{}:1:6 MD009 trailing whitespace\n", dirty.display())
        );
        assert_eq!(
            support::stderr(&output),
            format!("{}: not a regular file\n", fifo.display())
        );
        assert_eq!(output.status.code(), Some(2));
    }

    #[test]
    fn a_walked_directory_still_skips_non_regular_entries() {
        let dir = support::temp_dir("non-regular-walked");
        std::os::unix::fs::symlink("/dev/zero", dir.join("evil.md")).unwrap();
        make_fifo(&dir.join("p.md"));
        support::write_file(&dir, "ok.md", "clean\n");

        let output = support::run_lint_with_deadline(&[dir.to_str().unwrap()], DEADLINE);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(support::stdout(&output), "");
        assert_eq!(support::stderr(&output), "");
        assert_eq!(output.status.code(), Some(0));
    }
}
