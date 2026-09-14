//! Shared helpers for the `tests/cli_*.rs` integration tests, which run the built `lint` binary
//! directly (`std::process::Command` + `CARGO_BIN_EXE_lint`) rather than adding a dev-dependency
//! the architecture contract does not name.
//!
//! Each `tests/cli_*.rs` file compiles this module into its own separate test binary (S §14.1),
//! so any one binary that does not call every helper here is expected, not dead code.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{SystemTime, UNIX_EPOCH};

/// Runs the built `lint` binary with `args`, from the given working directory, and returns its
/// captured output.
pub fn run_lint(args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lint"))
        .args(args)
        .output()
        .expect("failed to execute the built lint binary")
}

/// Creates a fresh, empty temporary directory under the system temp directory, unique to this
/// process and call. The name folds in the process ID, the current timestamp, and a process-local
/// atomic counter, so two calls racing on the same clock tick within this test binary can never
/// collide (rework of F1: pid+nanos alone raced under parallel test threads).
pub fn temp_dir(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};

    static CALL_COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = CALL_COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before the Unix epoch")
        .as_nanos();
    dir.push(format!(
        "lint-cli-test-{label}-{}-{nanos}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

/// Writes `contents` to `dir.join(name)`, creating parent directories as needed.
pub fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    std::fs::write(&path, contents).expect("failed to write test fixture file");
    path
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("lint's stdout was not valid UTF-8")
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("lint's stderr was not valid UTF-8")
}
