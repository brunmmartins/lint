use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use lint_application::{ReadFault, SourceReader};

/// The largest file, in bytes, that is read. A larger file is rejected from its metadata before it
/// is opened, and a file that grows past this bound while it is read is rejected after reading at
/// most one byte more than the bound.
pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;

/// Reads a regular file's bytes from the real file system, within [`MAX_FILE_BYTES`], then
/// UTF-8-validates them.
///
/// Anything that is not a regular file, or a symlink to one, is rejected: a named pipe, a device,
/// or a directory is never read, and a named pipe or a symlink to one is never opened, so it cannot
/// block the run. The type is checked from the path before opening and again on the opened handle,
/// and the read itself stops one byte past the bound.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdSourceReader;

impl SourceReader for StdSourceReader {
    fn read(&self, path: &Path) -> Result<String, ReadFault> {
        // Follows symlinks and never opens the path, so a named pipe without a writer is judged
        // without blocking.
        let metadata = std::fs::metadata(path).map_err(|error| unreadable(path, &error))?;
        if !metadata.is_file() {
            return Err(not_regular_file(path));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(too_large(path));
        }

        let file = File::open(path).map_err(|error| unreadable(path, &error))?;
        let bytes = read_opened_file(&file, path)?;

        String::from_utf8(bytes).map_err(|_| ReadFault::InvalidUtf8 {
            path: path.to_path_buf(),
        })
    }
}

/// Reads an opened file within the size bound, after confirming the handle is a regular file. The
/// path may have been replaced between the first type check and the open; a replacement that is
/// not a regular file is rejected here, before any byte is read.
fn read_opened_file(file: &File, path: &Path) -> Result<Vec<u8>, ReadFault> {
    let metadata = file.metadata().map_err(|error| unreadable(path, &error))?;
    if !metadata.is_file() {
        return Err(not_regular_file(path));
    }

    match read_within_limit(file, metadata.len(), MAX_FILE_BYTES) {
        Ok(LimitedRead::Within(bytes)) => Ok(bytes),
        Ok(LimitedRead::OverLimit) => Err(too_large(path)),
        Err(error) => Err(unreadable(path, &error)),
    }
}

/// The result of reading a source through a byte limit.
#[derive(Debug, PartialEq, Eq)]
enum LimitedRead {
    /// Every byte of the source, which held no more than the limit.
    Within(Vec<u8>),
    /// The source held more bytes than the limit. Only one byte past the limit was read.
    OverLimit,
}

/// Reads `source` to its end, or until one byte past `limit`, whichever comes first.
///
/// `length_hint` sizes the buffer up front, capped at `limit`, so a truthful length allocates once
/// and a false one cannot allocate more than the limit before any byte arrives. A source that never
/// ends costs `limit + 1` bytes read.
fn read_within_limit<S: Read>(source: S, length_hint: u64, limit: u64) -> io::Result<LimitedRead> {
    let read_cap = limit.saturating_add(1);
    // The capacity is only a hint: when it does not fit in `usize`, the buffer grows as it fills.
    let capacity = usize::try_from(length_hint.min(limit).saturating_add(1)).unwrap_or(0);
    let mut bytes = Vec::with_capacity(capacity);
    source.take(read_cap).read_to_end(&mut bytes)?;

    let read = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if read > limit {
        Ok(LimitedRead::OverLimit)
    } else {
        Ok(LimitedRead::Within(bytes))
    }
}

fn unreadable(path: &Path, error: &io::Error) -> ReadFault {
    ReadFault::Unreadable {
        path: path.to_path_buf(),
        detail: error.to_string(),
    }
}

fn not_regular_file(path: &Path) -> ReadFault {
    ReadFault::NotRegularFile {
        path: path.to_path_buf(),
    }
}

fn too_large(path: &Path) -> ReadFault {
    ReadFault::TooLarge {
        path: path.to_path_buf(),
        limit_bytes: MAX_FILE_BYTES,
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
    fn reads_a_file_of_exactly_the_size_bound() {
        let dir = tempdir();
        let path = dir.join("bound.md");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_FILE_BYTES).unwrap();
        drop(file);

        let result = StdSourceReader.read(&path);

        std::fs::remove_dir_all(&dir).unwrap();
        let source = result.unwrap();
        assert_eq!(u64::try_from(source.len()).unwrap(), MAX_FILE_BYTES);
    }

    #[test]
    fn rejects_a_file_one_byte_over_the_size_bound() {
        let dir = tempdir();
        let path = dir.join("over.md");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_FILE_BYTES + 1).unwrap();
        drop(file);

        let result = StdSourceReader.read(&path);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            result,
            Err(ReadFault::TooLarge {
                path: path.clone(),
                limit_bytes: MAX_FILE_BYTES,
            })
        );
    }

    /// Counts the bytes pulled from the source it wraps.
    struct CountingSource<S> {
        inner: S,
        pulled: u64,
    }

    impl<S: Read> Read for CountingSource<S> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let count = self.inner.read(buf)?;
            self.pulled += u64::try_from(count).unwrap();
            Ok(count)
        }
    }

    #[test]
    fn a_source_that_never_ends_is_read_to_at_most_one_byte_past_the_bound() {
        // Stands in for a file that keeps growing after its size was checked.
        let mut source = CountingSource {
            inner: io::repeat(b'a'),
            pulled: 0,
        };

        let result = read_within_limit(&mut source, 0, MAX_FILE_BYTES).unwrap();

        assert_eq!(result, LimitedRead::OverLimit);
        assert!(
            source.pulled <= MAX_FILE_BYTES + 1,
            "pulled {}",
            source.pulled
        );
    }

    #[test]
    fn a_source_that_grows_past_its_reported_length_is_over_the_limit() {
        // The length hint claims a small file; the source yields more than the bound.
        let mut source = CountingSource {
            inner: io::repeat(b'a'),
            pulled: 0,
        };

        let result = read_within_limit(&mut source, 16, MAX_FILE_BYTES).unwrap();

        assert_eq!(result, LimitedRead::OverLimit);
        assert!(
            source.pulled <= MAX_FILE_BYTES + 1,
            "pulled {}",
            source.pulled
        );
    }

    #[test]
    fn a_source_of_exactly_the_bound_is_accepted() {
        let source = io::repeat(b'a').take(MAX_FILE_BYTES);

        let result = read_within_limit(source, MAX_FILE_BYTES, MAX_FILE_BYTES).unwrap();

        match result {
            LimitedRead::Within(bytes) => {
                assert_eq!(u64::try_from(bytes.len()).unwrap(), MAX_FILE_BYTES);
            }
            LimitedRead::OverLimit => panic!("a source of exactly the bound was rejected"),
        }
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

        // Not verifiable when the test runner is root: permission bits are bypassed for root on
        // most platforms, so the read succeeds instead of failing. Only assert the failure shape
        // when the read did fail.
        if let Err(fault) = result {
            assert!(matches!(fault, ReadFault::Unreadable { .. }));
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_character_device_is_not_a_regular_file() {
        let path = Path::new("/dev/null");

        let result = StdSourceReader.read(path);

        assert_eq!(
            result,
            Err(ReadFault::NotRegularFile {
                path: path.to_path_buf()
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_device_is_not_a_regular_file() {
        let dir = tempdir();
        let path = dir.join("evil.md");
        std::os::unix::fs::symlink("/dev/zero", &path).unwrap();

        let result = StdSourceReader.read(&path);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            result,
            Err(ReadFault::NotRegularFile { path: path.clone() })
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_directory_is_not_a_regular_file() {
        let dir = tempdir();
        let target = dir.join("target");
        std::fs::create_dir(&target).unwrap();
        let path = dir.join("dlink.md");
        std::os::unix::fs::symlink(&target, &path).unwrap();

        let result = StdSourceReader.read(&path);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            result,
            Err(ReadFault::NotRegularFile { path: path.clone() })
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_regular_file_is_read() {
        let dir = tempdir();
        let target = dir.join("real.md");
        std::fs::write(&target, "hello\n").unwrap();
        let path = dir.join("link.md");
        std::os::unix::fs::symlink(&target, &path).unwrap();

        let result = StdSourceReader.read(&path);

        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result.unwrap(), "hello\n");
    }

    #[cfg(unix)]
    #[test]
    fn an_opened_handle_that_is_not_a_regular_file_is_rejected_before_reading() {
        // An endless device stands in for a regular file replaced after the path was checked.
        // Reading from it would end in a size fault, so only a rejection before reading passes.
        let path = Path::new("/dev/zero");
        let file = File::open(path).unwrap();

        let result = read_opened_file(&file, path);

        assert_eq!(
            result,
            Err(ReadFault::NotRegularFile {
                path: path.to_path_buf()
            })
        );
    }

    /// Builds a fresh, unique temp directory for one test. The name folds in the process ID, the
    /// current timestamp, and a process-local atomic counter, so two calls racing on the same
    /// clock tick within this test binary can never collide (the process ID and timestamp alone raced
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
