use std::io::{self, Write};

use lint_application::{FileFindings, LintFault, ReportFault, Reporter, format_finding};

/// Writes findings as `path:line:col RULE message` lines to one stream and faults as lines to
/// another, as the run produces them.
///
/// Each file's findings are flushed before [`Reporter::report_findings`] returns, so a buffered
/// output never holds one file's lines while the next file is read, and a failure that surfaces
/// only on flush is still returned. A failed write to the fault stream is ignored.
#[derive(Debug)]
pub struct TextReporter<O: Write, E: Write> {
    out: O,
    err: E,
}

impl<O: Write, E: Write> TextReporter<O, E> {
    /// Builds a reporter that writes findings to `out` and faults to `err`.
    pub fn new(out: O, err: E) -> Self {
        Self { out, err }
    }

    #[cfg(test)]
    fn out(&self) -> &O {
        &self.out
    }

    #[cfg(test)]
    fn err(&self) -> &E {
        &self.err
    }
}

impl<O: Write, E: Write> Reporter for TextReporter<O, E> {
    fn report_findings(&mut self, file: &FileFindings) -> Result<(), ReportFault> {
        for finding in &file.findings {
            writeln!(self.out, "{}", format_finding(&file.path, finding))
                .map_err(output_failure)?;
        }
        self.out.flush().map_err(output_failure)
    }

    fn report_fault(&mut self, fault: &LintFault) {
        // A fault is only ever reported in a run that already exits with status 2, so a fault
        // stream that cannot be written loses the explanation but never the signal. Stopping or
        // panicking here would turn a reported problem into a worse one.
        let _ = writeln!(self.err, "{fault}");
    }

    fn finish(&mut self) -> Result<(), ReportFault> {
        self.out.flush().map_err(output_failure)
    }
}

/// Classifies a failed write: a reader that has gone away is a closed output, which needs no
/// explanation; anything else keeps the operating system's reason.
fn output_failure(error: io::Error) -> ReportFault {
    if error.kind() == io::ErrorKind::BrokenPipe {
        ReportFault::Closed
    } else {
        ReportFault::Unwritable {
            detail: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lint_domain::{Finding, FindingMessage, Location, RuleId};
    use std::io::BufWriter;
    use std::path::PathBuf;

    /// Fails every write with the given error.
    struct FailingWriter {
        error: fn() -> io::Error,
    }

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err((self.error)())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Accepts every write, and fails every flush with the given error.
    struct FlushFailingWriter {
        error: fn() -> io::Error,
    }

    impl Write for FlushFailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err((self.error)())
        }
    }

    fn broken_pipe() -> io::Error {
        io::Error::from(io::ErrorKind::BrokenPipe)
    }

    fn no_space() -> io::Error {
        // ENOSPC on Linux.
        io::Error::from_raw_os_error(28)
    }

    fn two_findings() -> FileFindings {
        let at = |line, col| Location::new(line, col).unwrap();
        FileFindings {
            path: PathBuf::from("doc.md"),
            findings: vec![
                Finding::new(RuleId::Md010, at(1, 1), FindingMessage::HardTab),
                Finding::new(RuleId::Md009, at(2, 2), FindingMessage::TrailingWhitespace),
            ],
        }
    }

    fn expected_lines(file: &FileFindings) -> String {
        file.findings
            .iter()
            .map(|finding| format!("{}\n", format_finding(&file.path, finding)))
            .collect()
    }

    #[test]
    fn a_broken_pipe_is_closed() {
        let mut reporter = TextReporter::new(FailingWriter { error: broken_pipe }, Vec::new());

        let result = reporter.report_findings(&two_findings());

        assert_eq!(result, Err(ReportFault::Closed));
    }

    #[test]
    fn another_write_error_is_unwritable_with_its_reason() {
        let mut reporter = TextReporter::new(FailingWriter { error: no_space }, Vec::new());

        let result = reporter.report_findings(&two_findings());

        assert_eq!(
            result,
            Err(ReportFault::Unwritable {
                detail: no_space().to_string()
            })
        );
        assert!(no_space().to_string().contains("os error 28"));
    }

    #[test]
    fn a_failure_only_at_flush_is_returned_from_report_findings() {
        let mut reporter = TextReporter::new(FlushFailingWriter { error: no_space }, Vec::new());

        let result = reporter.report_findings(&two_findings());

        assert_eq!(
            result,
            Err(ReportFault::Unwritable {
                detail: no_space().to_string()
            })
        );
    }

    #[test]
    fn a_failure_only_at_flush_is_returned_from_finish() {
        let mut reporter = TextReporter::new(FlushFailingWriter { error: no_space }, Vec::new());

        assert_eq!(
            reporter.finish(),
            Err(ReportFault::Unwritable {
                detail: no_space().to_string()
            })
        );

        let mut reporter = TextReporter::new(FlushFailingWriter { error: broken_pipe }, Vec::new());

        assert_eq!(reporter.finish(), Err(ReportFault::Closed));
    }

    #[test]
    fn findings_are_flushed_before_report_findings_returns() {
        // A buffer far larger than the lines, so they reach the inner writer only by a flush.
        let out = BufWriter::with_capacity(64 * 1024, Vec::new());
        let mut reporter = TextReporter::new(out, Vec::new());
        let file = two_findings();

        reporter.report_findings(&file).unwrap();

        assert_eq!(
            String::from_utf8(reporter.out().get_ref().clone()).unwrap(),
            expected_lines(&file)
        );
        assert_eq!(
            expected_lines(&file),
            "doc.md:1:1 MD010 hard tab\ndoc.md:2:2 MD009 trailing whitespace\n"
        );
    }

    #[test]
    fn a_fault_is_written_as_one_line_to_the_fault_stream() {
        let mut reporter = TextReporter::new(Vec::new(), Vec::new());

        reporter.report_fault(&LintFault::NotFound {
            path: PathBuf::from("missing.md"),
        });

        assert!(reporter.out().is_empty());
        assert_eq!(
            String::from_utf8(reporter.err().clone()).unwrap(),
            "missing.md: no such file or directory\n"
        );
    }

    #[test]
    fn a_failing_error_stream_is_ignored() {
        let mut reporter = TextReporter::new(Vec::new(), FailingWriter { error: broken_pipe });

        reporter.report_fault(&LintFault::NotFound {
            path: PathBuf::from("missing.md"),
        });
        let file = two_findings();
        reporter.report_findings(&file).unwrap();
        reporter.finish().unwrap();

        assert_eq!(
            String::from_utf8(reporter.out().clone()).unwrap(),
            expected_lines(&file)
        );
    }
}
