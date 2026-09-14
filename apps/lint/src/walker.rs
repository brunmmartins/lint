use std::path::{Path, PathBuf};

use lint_application::{WalkFault, Walker};

/// Walks the real file system without ever following a symlink as a directory (ADR-0005). A
/// symlink loop is therefore structurally impossible to form during traversal, not merely bounded.
///
/// Traversal is iterative (an explicit `Vec<PathBuf>` stack), never recursive, so directory depth
/// cannot exhaust the native call stack.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdWalker;

impl Walker for StdWalker {
    fn resolve(&self, input: &Path) -> Result<Vec<PathBuf>, WalkFault> {
        let metadata = std::fs::symlink_metadata(input).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                WalkFault::NotFound {
                    path: input.to_path_buf(),
                }
            } else {
                WalkFault::Unreadable {
                    path: input.to_path_buf(),
                    detail: error.to_string(),
                }
            }
        })?;

        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            walk_directory(input)
        } else {
            // An explicit file path (including a symlink to a file) is always linted regardless
            // of extension, since naming it directly is explicit user intent (ADR-0005).
            Ok(vec![input.to_path_buf()])
        }
    }
}

/// Iterative depth-first walk in sorted-by-name order (ADR-0005). Never follows a symlink as a
/// directory: `symlink_metadata` (never `metadata`) decides whether to recurse.
fn walk_directory(root: &Path) -> Result<Vec<PathBuf>, WalkFault> {
    let mut results = Vec::new();
    let mut stack = sorted_children(root)?;
    // Reverse so `pop()` yields entries in ascending sorted order; each directory popped later
    // pushes its own sorted-and-reversed children on top, which keeps the overall order a proper
    // sorted depth-first pre-order traversal.
    stack.reverse();

    while let Some(entry) = stack.pop() {
        let metadata = match std::fs::symlink_metadata(&entry) {
            Ok(metadata) => metadata,
            // The entry vanished between listing its parent and stat-ing it (a benign race, not
            // an input this card's ACs exercise); skip it rather than fail the whole walk.
            Err(_) => continue,
        };

        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let mut children = sorted_children(&entry)?;
            children.reverse();
            stack.extend(children);
        } else if metadata.is_file() && is_markdown_extension(&entry) {
            results.push(entry);
        }
        // A symlink (to a file or a directory) discovered during the walk, or a non-Markdown
        // file, is neither recursed into nor collected.
    }

    Ok(results)
}

fn sorted_children(dir: &Path) -> Result<Vec<PathBuf>, WalkFault> {
    let entries = std::fs::read_dir(dir).map_err(|error| WalkFault::Unreadable {
        path: dir.to_path_buf(),
        detail: error.to_string(),
    })?;

    let mut children = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| WalkFault::Unreadable {
            path: dir.to_path_buf(),
            detail: error.to_string(),
        })?;
        children.push(entry.path());
    }
    children.sort();
    Ok(children)
}

fn is_markdown_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Builds a fresh, unique temp directory for one test. The name folds in the process ID, the
    /// current timestamp, and a process-local atomic counter, so two calls racing on the same
    /// clock tick within this test binary can never collide (rework of F1: pid+nanos alone raced
    /// under parallel test threads).
    fn tempdir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};

        static CALL_COUNTER: AtomicU64 = AtomicU64::new(0);
        let unique = CALL_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "lint-walker-test-{}-{}-{unique}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolving_a_nonexistent_path_is_not_found() {
        let dir = tempdir();
        let missing = dir.join("missing.md");

        let result = StdWalker.resolve(&missing);

        assert!(matches!(result, Err(WalkFault::NotFound { .. })));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolving_a_single_file_returns_just_that_file_regardless_of_extension() {
        let dir = tempdir();
        let file = dir.join("notes.txt");
        std::fs::write(&file, "hello").unwrap();

        let result = StdWalker.resolve(&file).unwrap();

        assert_eq!(result, vec![file.clone()]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_directory_with_no_markdown_files_resolves_to_an_empty_list() {
        let dir = tempdir();
        std::fs::write(dir.join("readme.txt"), "hello").unwrap();

        let result = StdWalker.resolve(&dir).unwrap();

        assert!(result.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_directory_is_walked_recursively_in_sorted_order() {
        let dir = tempdir();
        std::fs::write(dir.join("b.md"), "b").unwrap();
        std::fs::create_dir_all(dir.join("nested")).unwrap();
        std::fs::write(dir.join("nested/c.md"), "c").unwrap();
        std::fs::write(dir.join("a.md"), "a").unwrap();
        std::fs::write(dir.join("ignore.txt"), "ignored").unwrap();

        let result = StdWalker.resolve(&dir).unwrap();

        assert_eq!(
            result,
            vec![dir.join("a.md"), dir.join("b.md"), dir.join("nested/c.md")]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn markdown_extension_matching_is_case_insensitive() {
        let dir = tempdir();
        std::fs::write(dir.join("upper.MD"), "x").unwrap();
        std::fs::write(dir.join("mixed.Markdown"), "x").unwrap();

        let result = StdWalker.resolve(&dir).unwrap();

        assert_eq!(result.len(), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_directory_is_never_recursed_into() {
        use std::os::unix::fs::symlink;

        let dir = tempdir();
        std::fs::create_dir_all(dir.join("real")).unwrap();
        std::fs::write(dir.join("real/target.md"), "x").unwrap();
        // A self-referential symlink loop: `dir/loop` points back at `dir`.
        symlink(&dir, dir.join("loop")).unwrap();

        let result = StdWalker.resolve(&dir).unwrap();

        assert_eq!(result, vec![dir.join("real/target.md")]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
