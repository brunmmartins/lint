use std::fmt;
use std::path::PathBuf;

/// A fault from resolving one CLI input into file paths (the [`Walker`](crate::Walker) port).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalkFault {
    /// The path does not exist on disk.
    NotFound {
        /// The offending path.
        path: PathBuf,
    },
    /// The path exists but a directory entry under it could not be listed.
    Unreadable {
        /// The offending path.
        path: PathBuf,
        /// A human-readable detail (never file contents).
        detail: String,
    },
}

/// A fault from reading one file's contents (the [`SourceReader`](crate::SourceReader) port).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadFault {
    /// The file exists but could not be read (for example, permission denied).
    Unreadable {
        /// The offending path.
        path: PathBuf,
        /// A human-readable detail (never file contents).
        detail: String,
    },
    /// The file is larger than the size bound enforced before allocating a read buffer.
    TooLarge {
        /// The offending path.
        path: PathBuf,
        /// The bound that was exceeded, in bytes.
        limit_bytes: u64,
    },
    /// The file's bytes are not valid UTF-8.
    InvalidUtf8 {
        /// The offending path.
        path: PathBuf,
    },
    /// The path is not a regular file, or a symlink to one: for example a named pipe, a device, or
    /// a directory. It was not read.
    NotRegularFile {
        /// The offending path.
        path: PathBuf,
    },
}

/// A failure to write results to the output (the [`Reporter`](crate::Reporter) port). Either
/// variant stops the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportFault {
    /// The output's reader has gone away, as when a pipe into `head` closes. Nothing more can be
    /// delivered, and nothing needs explaining to a reader who has left.
    Closed,
    /// Any other failure to write or flush the output.
    Unwritable {
        /// The operating system's description of the failure (never file contents).
        detail: String,
    },
}

/// The unified fault taxonomy handed to [`Reporter::report_fault`](crate::Reporter::report_fault)
/// and folded into the exit-code decision. Never a raw `io::Error`: ports translate at their own
/// boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintFault {
    /// The path does not exist on disk.
    NotFound {
        /// The offending path.
        path: PathBuf,
    },
    /// The path exists but could not be read.
    Unreadable {
        /// The offending path.
        path: PathBuf,
        /// A human-readable detail (never file contents).
        detail: String,
    },
    /// The file is larger than the enforced size bound.
    TooLarge {
        /// The offending path.
        path: PathBuf,
        /// The bound that was exceeded, in bytes.
        limit_bytes: u64,
    },
    /// The file's bytes are not valid UTF-8.
    InvalidUtf8 {
        /// The offending path.
        path: PathBuf,
    },
    /// The path is not a regular file, or a symlink to one. It was not read.
    NotRegularFile {
        /// The offending path.
        path: PathBuf,
    },
    /// Standard output could not be written, so the run stopped.
    OutputUnwritable {
        /// The operating system's description of the failure (never file contents).
        detail: String,
    },
}

impl From<WalkFault> for LintFault {
    fn from(fault: WalkFault) -> Self {
        match fault {
            WalkFault::NotFound { path } => LintFault::NotFound { path },
            WalkFault::Unreadable { path, detail } => LintFault::Unreadable { path, detail },
        }
    }
}

impl From<ReadFault> for LintFault {
    fn from(fault: ReadFault) -> Self {
        match fault {
            ReadFault::Unreadable { path, detail } => LintFault::Unreadable { path, detail },
            ReadFault::TooLarge { path, limit_bytes } => LintFault::TooLarge { path, limit_bytes },
            ReadFault::InvalidUtf8 { path } => LintFault::InvalidUtf8 { path },
            ReadFault::NotRegularFile { path } => LintFault::NotRegularFile { path },
        }
    }
}

impl fmt::Display for LintFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LintFault::NotFound { path } => {
                write!(f, "{}: no such file or directory", path.display())
            }
            LintFault::Unreadable { path, detail } => {
                write!(f, "{}: cannot be read ({detail})", path.display())
            }
            LintFault::TooLarge { path, limit_bytes } => write!(
                f,
                "{}: exceeds the maximum size of {limit_bytes} bytes",
                path.display()
            ),
            LintFault::InvalidUtf8 { path } => write!(f, "{}: not valid UTF-8", path.display()),
            LintFault::NotRegularFile { path } => {
                write!(f, "{}: not a regular file", path.display())
            }
            LintFault::OutputUnwritable { detail } => {
                write!(f, "standard output: cannot be written ({detail})")
            }
        }
    }
}

impl std::error::Error for LintFault {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_fault_not_found_converts_to_lint_fault() {
        let fault = WalkFault::NotFound {
            path: PathBuf::from("missing.md"),
        };
        assert_eq!(
            LintFault::from(fault),
            LintFault::NotFound {
                path: PathBuf::from("missing.md")
            }
        );
    }

    #[test]
    fn read_fault_too_large_converts_to_lint_fault() {
        let fault = ReadFault::TooLarge {
            path: PathBuf::from("huge.md"),
            limit_bytes: 10,
        };
        assert_eq!(
            LintFault::from(fault),
            LintFault::TooLarge {
                path: PathBuf::from("huge.md"),
                limit_bytes: 10
            }
        );
    }

    #[test]
    fn read_fault_not_regular_file_converts_to_lint_fault() {
        let fault = ReadFault::NotRegularFile {
            path: PathBuf::from("pipe.md"),
        };
        assert_eq!(
            LintFault::from(fault),
            LintFault::NotRegularFile {
                path: PathBuf::from("pipe.md")
            }
        );
    }

    #[test]
    fn not_regular_file_names_the_path() {
        let fault = LintFault::NotRegularFile {
            path: PathBuf::from("docs/evil.md"),
        };
        assert_eq!(fault.to_string(), "docs/evil.md: not a regular file");
    }

    #[test]
    fn output_unwritable_names_standard_output_and_the_reason() {
        let fault = LintFault::OutputUnwritable {
            detail: "No space left on device (os error 28)".to_string(),
        };
        assert_eq!(
            fault.to_string(),
            "standard output: cannot be written (No space left on device (os error 28))"
        );
    }

    #[test]
    fn display_never_echoes_file_contents() {
        let fault = LintFault::Unreadable {
            path: PathBuf::from("secret.md"),
            detail: "permission denied".to_string(),
        };
        let rendered = fault.to_string();
        assert!(rendered.contains("secret.md"));
        assert!(rendered.contains("permission denied"));
    }
}
