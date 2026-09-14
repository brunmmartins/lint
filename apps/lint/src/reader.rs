use std::path::Path;

use lint_application::{ReadFault, SourceReader};

/// The size bound enforced before any read buffer is allocated (K §22.2: "validation happens
/// before allocation"). Fixed by the architecture contract's runtime-behavior section.
pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;

/// Reads a file's bytes from the real file system, enforcing [`MAX_FILE_BYTES`] via
/// `std::fs::metadata` before the read, then UTF-8-validating the result.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdSourceReader;

impl SourceReader for StdSourceReader {
    fn read(&self, path: &Path) -> Result<String, ReadFault> {
        let metadata = std::fs::metadata(path).map_err(|error| ReadFault::Unreadable {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })?;

        if metadata.len() > MAX_FILE_BYTES {
            return Err(ReadFault::TooLarge {
                path: path.to_path_buf(),
                limit_bytes: MAX_FILE_BYTES,
            });
        }

        let bytes = std::fs::read(path).map_err(|error| ReadFault::Unreadable {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })?;

        String::from_utf8(bytes).map_err(|_| ReadFault::InvalidUtf8 {
            path: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_a_small_utf8_file() {
        let dir = tempdir();
        let path = dir.join("clean.md");
        std::fs::write(&path, "hello\n").unwrap();

        let result = StdSourceReader.read(&path);

        assert_eq!(result.unwrap(), "hello\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_a_file_over_the_size_bound() {
        let dir = tempdir();
        let path = dir.join("huge.md");
        let mut file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_FILE_BYTES + 1).unwrap();
        file.flush().unwrap();

        let result = StdSourceReader.read(&path);

        assert!(matches!(result, Err(ReadFault::TooLarge { .. })));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_invalid_utf8() {
        let dir = tempdir();
        let path = dir.join("invalid.md");
        std::fs::write(&path, [0x66, 0x6f, 0x80, 0x6f]).unwrap();

        let result = StdSourceReader.read(&path);

        assert!(matches!(result, Err(ReadFault::InvalidUtf8 { .. })));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reports_a_missing_file_as_unreadable() {
        let dir = tempdir();
        let path = dir.join("missing.md");

        let result = StdSourceReader.read(&path);

        assert!(matches!(result, Err(ReadFault::Unreadable { .. })));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn reports_a_permission_denied_file_as_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir();
        let path = dir.join("secret.md");
        std::fs::write(&path, "secret\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        let result = StdSourceReader.read(&path);

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        // Left unverified when the test runner is root: permission bits are bypassed for root on
        // most platforms, so the read succeeds instead of failing (architecture contract, "Left
        // unverified"). Only assert the failure shape when the read did fail.
        if let Err(fault) = result {
            assert!(matches!(fault, ReadFault::Unreadable { .. }));
        }
    }

    /// Builds a fresh, unique temp directory for one test. The name folds in the process ID, the
    /// current timestamp, and a process-local atomic counter, so two calls racing on the same
    /// clock tick within this test binary can never collide (rework of F1: pid+nanos alone raced
    /// under parallel test threads).
    fn tempdir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};

        static CALL_COUNTER: AtomicU64 = AtomicU64::new(0);
        let unique = CALL_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "lint-reader-test-{}-{}-{unique}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
