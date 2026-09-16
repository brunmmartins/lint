use std::path::PathBuf;

use lint_domain::{Finding, RULES};

use crate::{
    LintFault, MarkdownParser, ReportFault, Reporter, SourceReader, Walker, decide_exit_code,
};

/// The findings of one file, with its path stored once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFindings {
    /// The file the findings came from.
    pub path: PathBuf,
    /// The file's findings, sorted by location and then by rule ID.
    pub findings: Vec<Finding>,
}

/// What a [`run_lint`] call reported, as counts. The findings and faults themselves went to the
/// [`Reporter`] as they were produced.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintOutcome {
    /// How many files were handed to [`Reporter::report_findings`], including one whose write
    /// failed.
    pub files_with_findings: usize,
    /// How many walk and read faults were handed to [`Reporter::report_fault`].
    pub faults: usize,
    /// The output failure that stopped the run before every input was processed, if any.
    pub stopped: Option<ReportFault>,
}

impl LintOutcome {
    /// The process exit status for this outcome: `2` when any fault occurred or the output failed,
    /// otherwise `1` when any file had findings, otherwise `0`.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        decide_exit_code(
            self.faults > 0 || self.stopped.is_some(),
            self.files_with_findings > 0,
        )
    }
}

/// Walks, reads, parses, and checks every path in `inputs`, in order, handing results to
/// `reporter` as they are produced.
///
/// Each file's findings reach the reporter before the next path is resolved or read, and each
/// fault reaches it when it occurs, so nothing from one file is held while the next is processed.
/// A fault on one input (not found, unreadable, not a regular file, too large, invalid UTF-8) is
/// reported and that input is skipped; every other input is still processed, so one bad path
/// never hides the problems in the others. After the last input, the reporter is finished.
///
/// When the reporter fails to deliver findings or to finish, the run stops at once: no further
/// path is resolved or read, and the failure is recorded in [`LintOutcome::stopped`]. A closed
/// output is not reported further. Any other output failure is reported once, as
/// [`LintFault::OutputUnwritable`].
pub fn run_lint<W, R, P, O>(
    inputs: &[PathBuf],
    walker: &W,
    reader: &R,
    parser: &P,
    reporter: &mut O,
) -> LintOutcome
where
    W: Walker,
    R: SourceReader,
    P: MarkdownParser,
    O: Reporter,
{
    let mut outcome = LintOutcome::default();

    for input in inputs {
        let files = match walker.resolve(input) {
            Ok(files) => files,
            Err(fault) => {
                outcome.faults = outcome.faults.saturating_add(1);
                reporter.report_fault(&LintFault::from(fault));
                continue;
            }
        };
        for file in files {
            if let Err(failure) = lint_one_file(file, reader, parser, reporter, &mut outcome) {
                return stop(outcome, reporter, failure);
            }
        }
    }

    match reporter.finish() {
        Ok(()) => outcome,
        Err(failure) => stop(outcome, reporter, failure),
    }
}

/// Reads, checks, and reports one file. Its source, document, and findings are all dropped when
/// this returns, before the next file is read.
fn lint_one_file<R, P, O>(
    file: PathBuf,
    reader: &R,
    parser: &P,
    reporter: &mut O,
    outcome: &mut LintOutcome,
) -> Result<(), ReportFault>
where
    R: SourceReader,
    P: MarkdownParser,
    O: Reporter,
{
    let findings = match reader.read(&file) {
        Ok(source) => check_source(&source, parser),
        Err(fault) => {
            outcome.faults = outcome.faults.saturating_add(1);
            reporter.report_fault(&LintFault::from(fault));
            return Ok(());
        }
    };
    if findings.is_empty() {
        return Ok(());
    }

    outcome.files_with_findings = outcome.files_with_findings.saturating_add(1);
    reporter.report_findings(&FileFindings {
        path: file,
        findings,
    })
}

/// Runs every rule over one source and returns its findings in output order.
fn check_source<P: MarkdownParser>(source: &str, parser: &P) -> Vec<Finding> {
    let document = parser.parse(source);
    let mut findings: Vec<Finding> = RULES
        .iter()
        .flat_map(|rule| rule.check(&document))
        .collect();
    sort_file_findings(&mut findings);
    findings
}

/// Ends a run the output could not keep up with: records why, and reports an unwritable output
/// once. A closed output is not reported, since its reader has already gone.
fn stop<O: Reporter>(
    mut outcome: LintOutcome,
    reporter: &mut O,
    failure: ReportFault,
) -> LintOutcome {
    if let ReportFault::Unwritable { detail } = &failure {
        reporter.report_fault(&LintFault::OutputUnwritable {
            detail: detail.clone(),
        });
    }
    outcome.stopped = Some(failure);
    outcome
}

/// Orders one file's findings by location, then by rule ID for findings at the same location.
fn sort_file_findings(findings: &mut [Finding]) {
    findings.sort_by_key(|finding| (finding.location(), finding.rule()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReadFault, WalkFault};
    use lint_domain::{Location, RuleId};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;
    use std::rc::Rc;

    /// One call into a port, in the order the use case made it.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Call {
        Resolve(PathBuf),
        Read(PathBuf),
        Findings(FileFindings),
        Fault(LintFault),
        Finish,
    }

    /// The call log every fake appends to, so the interleaving across ports is observable.
    type Log = Rc<RefCell<Vec<Call>>>;

    /// Resolves an input to the configured result, or to the input itself when none is configured.
    struct FakeWalker {
        resolutions: HashMap<PathBuf, Result<Vec<PathBuf>, WalkFault>>,
        log: Log,
    }

    impl Walker for FakeWalker {
        fn resolve(&self, input: &Path) -> Result<Vec<PathBuf>, WalkFault> {
            self.log
                .borrow_mut()
                .push(Call::Resolve(input.to_path_buf()));
            self.resolutions
                .get(input)
                .cloned()
                .unwrap_or_else(|| Ok(vec![input.to_path_buf()]))
        }
    }

    /// Returns the configured source for a path, or an empty source when none is configured.
    struct FakeReader {
        sources: HashMap<PathBuf, Result<String, ReadFault>>,
        log: Log,
    }

    impl SourceReader for FakeReader {
        fn read(&self, path: &Path) -> Result<String, ReadFault> {
            self.log.borrow_mut().push(Call::Read(path.to_path_buf()));
            self.sources
                .get(path)
                .cloned()
                .unwrap_or_else(|| Ok(String::new()))
        }
    }

    struct FakeParser;

    impl MarkdownParser for FakeParser {
        fn parse(&self, source: &str) -> lint_domain::Document {
            lint_domain::Document::from_source(source)
        }
    }

    /// Records every source it was asked to parse, so ordering assertions can rely on it.
    struct RecordingParser {
        calls: RefCell<Vec<String>>,
    }

    impl MarkdownParser for RecordingParser {
        fn parse(&self, source: &str) -> lint_domain::Document {
            self.calls.borrow_mut().push(source.to_string());
            lint_domain::Document::from_source(source)
        }
    }

    /// Records every call, and fails the chosen `report_findings` call or `finish` on request.
    struct FakeReporter {
        log: Log,
        findings_calls: usize,
        fail_findings_call: Option<(usize, ReportFault)>,
        fail_finish: Option<ReportFault>,
    }

    impl Reporter for FakeReporter {
        fn report_findings(&mut self, file: &FileFindings) -> Result<(), ReportFault> {
            self.log.borrow_mut().push(Call::Findings(file.clone()));
            self.findings_calls += 1;
            match &self.fail_findings_call {
                Some((call, failure)) if *call == self.findings_calls => Err(failure.clone()),
                _ => Ok(()),
            }
        }

        fn report_fault(&mut self, fault: &LintFault) {
            self.log.borrow_mut().push(Call::Fault(fault.clone()));
        }

        fn finish(&mut self) -> Result<(), ReportFault> {
            self.log.borrow_mut().push(Call::Finish);
            self.fail_finish.clone().map_or(Ok(()), Err)
        }
    }

    /// What the fakes return for one run.
    #[derive(Default)]
    struct Scenario {
        resolutions: HashMap<PathBuf, Result<Vec<PathBuf>, WalkFault>>,
        sources: HashMap<PathBuf, Result<String, ReadFault>>,
        fail_findings_call: Option<(usize, ReportFault)>,
        fail_finish: Option<ReportFault>,
    }

    impl Scenario {
        fn resolve(mut self, input: &str, files: &[&str]) -> Self {
            let files = files.iter().map(PathBuf::from).collect();
            self.resolutions.insert(PathBuf::from(input), Ok(files));
            self
        }

        fn resolve_fault(mut self, input: &str, fault: WalkFault) -> Self {
            self.resolutions.insert(PathBuf::from(input), Err(fault));
            self
        }

        fn source(mut self, path: &str, text: &str) -> Self {
            self.sources
                .insert(PathBuf::from(path), Ok(text.to_string()));
            self
        }

        fn read_fault(mut self, path: &str, fault: ReadFault) -> Self {
            self.sources.insert(PathBuf::from(path), Err(fault));
            self
        }

        /// Fails the `call`-th `report_findings` call (counting from 1) with `failure`.
        fn fail_findings(mut self, call: usize, failure: ReportFault) -> Self {
            self.fail_findings_call = Some((call, failure));
            self
        }

        fn fail_finish(mut self, failure: ReportFault) -> Self {
            self.fail_finish = Some(failure);
            self
        }

        fn run(self, inputs: &[&str]) -> (LintOutcome, Vec<Call>) {
            self.run_with_parser(inputs, &FakeParser)
        }

        fn run_with_parser<P: MarkdownParser>(
            self,
            inputs: &[&str],
            parser: &P,
        ) -> (LintOutcome, Vec<Call>) {
            let log = Log::default();
            let walker = FakeWalker {
                resolutions: self.resolutions,
                log: Rc::clone(&log),
            };
            let reader = FakeReader {
                sources: self.sources,
                log: Rc::clone(&log),
            };
            let mut reporter = FakeReporter {
                log: Rc::clone(&log),
                findings_calls: 0,
                fail_findings_call: self.fail_findings_call,
                fail_finish: self.fail_finish,
            };
            let inputs: Vec<PathBuf> = inputs.iter().map(PathBuf::from).collect();

            let outcome = run_lint(&inputs, &walker, &reader, parser, &mut reporter);

            drop((walker, reader, reporter));
            let calls = Rc::try_unwrap(log).unwrap().into_inner();
            (outcome, calls)
        }
    }

    /// Renders the call log as short lines, so an assertion reads as the expected sequence.
    fn trace(calls: &[Call]) -> Vec<String> {
        calls
            .iter()
            .map(|call| match call {
                Call::Resolve(path) => format!("resolve {}", path.display()),
                Call::Read(path) => format!("read {}", path.display()),
                Call::Findings(file) => format!("findings {}", file.path.display()),
                Call::Fault(fault) => format!("fault {fault}"),
                Call::Finish => "finish".to_string(),
            })
            .collect()
    }

    /// Every `FileFindings` handed to the reporter, in order.
    fn reported(calls: &[Call]) -> Vec<&FileFindings> {
        calls
            .iter()
            .filter_map(|call| match call {
                Call::Findings(file) => Some(file),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn clean_file_produces_no_findings_and_no_faults() {
        let (outcome, calls) = Scenario::default()
            .source("clean.md", "clean\n")
            .run(&["clean.md"]);

        assert_eq!(outcome, LintOutcome::default());
        assert_eq!(outcome.exit_code(), 0);
        assert_eq!(
            trace(&calls),
            ["resolve clean.md", "read clean.md", "finish"]
        );
    }

    #[test]
    fn a_fault_on_one_input_does_not_stop_the_others() {
        let (outcome, calls) = Scenario::default()
            .resolve_fault(
                "missing.md",
                WalkFault::NotFound {
                    path: PathBuf::from("missing.md"),
                },
            )
            .source("dirty.md", "trail  \n")
            .run(&["missing.md", "dirty.md"]);

        assert_eq!(outcome.faults, 1);
        assert_eq!(outcome.files_with_findings, 1);
        assert_eq!(outcome.exit_code(), 2);
        let files = reported(&calls);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("dirty.md"));
    }

    #[test]
    fn findings_preserve_input_order_then_walk_order() {
        let (outcome, calls) = Scenario::default()
            .resolve("dir", &["dir/a.md", "dir/b.md"])
            .source("dir/a.md", "trail  \n")
            .source("dir/b.md", "trail  \n")
            .run(&["dir"]);

        assert_eq!(outcome.files_with_findings, 2);
        assert_eq!(outcome.exit_code(), 1);
        let files = reported(&calls);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, PathBuf::from("dir/a.md"));
        assert_eq!(files[1].path, PathBuf::from("dir/b.md"));
    }

    #[test]
    fn findings_within_one_file_are_sorted_by_line_and_column() {
        // Missing trailing newline (reported at the last line) and a trailing-space line earlier
        // in the file: independent locations, MD009 before MD047 by position.
        let (_, calls) = Scenario::default()
            .source("both.md", "trail  \nclean")
            .run(&["both.md"]);

        let findings = &reported(&calls)[0].findings;
        assert_eq!(findings.len(), 2);
        assert!(findings[0].location() < findings[1].location());
    }

    #[test]
    fn same_location_findings_sort_by_rule_id() {
        use lint_domain::FindingMessage;

        let at = |line, col| Location::new(line, col).unwrap();
        let mut findings = vec![
            Finding::new(
                RuleId::Md047,
                at(3, 1),
                FindingMessage::MultipleTrailingNewlines,
            ),
            Finding::new(RuleId::Md010, at(1, 4), FindingMessage::HardTab),
            Finding::new(
                RuleId::Md012,
                at(3, 1),
                FindingMessage::MultipleBlankLines {
                    maximum: 1,
                    found: 2,
                },
            ),
            Finding::new(RuleId::Md009, at(1, 4), FindingMessage::TrailingWhitespace),
        ];

        sort_file_findings(&mut findings);

        let order: Vec<(Location, RuleId)> = findings
            .iter()
            .map(|finding| (finding.location(), finding.rule()))
            .collect();
        assert_eq!(
            order,
            [
                (at(1, 4), RuleId::Md009),
                (at(1, 4), RuleId::Md010),
                (at(3, 1), RuleId::Md012),
                (at(3, 1), RuleId::Md047),
            ]
        );

        // Through the use case as well: a trailing tab gives MD009 and MD010 at one location.
        let (_, calls) = Scenario::default()
            .source("tab.md", "abc\t\n")
            .run(&["tab.md"]);

        let rules: Vec<RuleId> = reported(&calls)[0]
            .findings
            .iter()
            .map(Finding::rule)
            .collect();
        assert_eq!(rules, [RuleId::Md009, RuleId::Md010]);
    }

    #[test]
    fn a_file_without_findings_gets_no_entry() {
        let (outcome, calls) = Scenario::default()
            .source("clean.md", "clean\n")
            .source("dirty.md", "a\tb\n")
            .run(&["clean.md", "dirty.md"]);

        assert_eq!(outcome.files_with_findings, 1);
        let files = reported(&calls);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("dirty.md"));
        assert_eq!(files[0].findings.len(), 1);
    }

    #[test]
    fn every_resolved_file_is_parsed_exactly_once() {
        let parser = RecordingParser {
            calls: RefCell::new(Vec::new()),
        };

        Scenario::default()
            .resolve("dir", &["dir/a.md", "dir/b.md"])
            .source("dir/a.md", "a\n")
            .source("dir/b.md", "b\n")
            .run_with_parser(&["dir"], &parser);

        assert_eq!(
            parser.calls.into_inner(),
            vec!["a\n".to_string(), "b\n".to_string()]
        );
    }

    #[test]
    fn each_files_findings_reach_the_reporter_before_the_next_file_is_read() {
        // A hard tab on line 1 and trailing spaces on line 2: the rules report MD009 (line 2)
        // before MD010 (line 1), so each file's findings arrive only if they were sorted first.
        let out_of_rule_order = "\tx\ny  \n";
        let (outcome, calls) = Scenario::default()
            .resolve("dir", &["dir/a.md", "dir/b.md", "dir/c.md"])
            .source("dir/a.md", out_of_rule_order)
            .source("dir/b.md", out_of_rule_order)
            .source("dir/c.md", out_of_rule_order)
            .run(&["dir"]);

        assert_eq!(
            trace(&calls),
            [
                "resolve dir",
                "read dir/a.md",
                "findings dir/a.md",
                "read dir/b.md",
                "findings dir/b.md",
                "read dir/c.md",
                "findings dir/c.md",
                "finish",
            ]
        );
        let at = |line, col| Location::new(line, col).unwrap();
        for file in reported(&calls) {
            let order: Vec<(Location, RuleId)> = file
                .findings
                .iter()
                .map(|finding| (finding.location(), finding.rule()))
                .collect();
            assert_eq!(
                order,
                [(at(1, 1), RuleId::Md010), (at(2, 2), RuleId::Md009)]
            );
        }
        assert_eq!(outcome.files_with_findings, 3);
        assert_eq!(outcome.exit_code(), 1);
    }

    #[test]
    fn faults_reach_the_reporter_when_they_occur_in_input_order() {
        let (outcome, calls) = Scenario::default()
            .source("a.md", "trail  \n")
            .resolve_fault(
                "missing.md",
                WalkFault::NotFound {
                    path: PathBuf::from("missing.md"),
                },
            )
            .resolve("dir", &["dir/bad.md", "dir/c.md"])
            .read_fault(
                "dir/bad.md",
                ReadFault::InvalidUtf8 {
                    path: PathBuf::from("dir/bad.md"),
                },
            )
            .source("dir/c.md", "trail  \n")
            .run(&["a.md", "missing.md", "dir"]);

        assert_eq!(
            trace(&calls),
            [
                "resolve a.md",
                "read a.md",
                "findings a.md",
                "resolve missing.md",
                "fault missing.md: no such file or directory",
                "resolve dir",
                "read dir/bad.md",
                "fault dir/bad.md: not valid UTF-8",
                "read dir/c.md",
                "findings dir/c.md",
                "finish",
            ]
        );
        assert_eq!(outcome.faults, 2);
        assert_eq!(outcome.files_with_findings, 2);
        assert_eq!(outcome.stopped, None);
        assert_eq!(outcome.exit_code(), 2);
    }

    #[test]
    fn a_closed_reporter_stops_the_run_before_any_further_path_is_resolved_or_read() {
        let (outcome, calls) = Scenario::default()
            .source("a.md", "trail  \n")
            .source("b.md", "trail  \n")
            .fail_findings(1, ReportFault::Closed)
            .run(&["a.md", "b.md"]);

        assert_eq!(
            trace(&calls),
            ["resolve a.md", "read a.md", "findings a.md"]
        );
        assert_eq!(outcome.stopped, Some(ReportFault::Closed));
        assert_eq!(outcome.faults, 0);
        assert_eq!(outcome.files_with_findings, 1);
        assert_eq!(outcome.exit_code(), 2);
    }

    #[test]
    fn a_closed_reporter_stops_the_run_before_the_next_file_in_the_same_directory_is_read() {
        let (outcome, calls) = Scenario::default()
            .resolve("dir", &["dir/a.md", "dir/b.md"])
            .source("dir/a.md", "trail  \n")
            .source("dir/b.md", "trail  \n")
            .fail_findings(1, ReportFault::Closed)
            .run(&["dir", "later.md"]);

        assert_eq!(
            trace(&calls),
            ["resolve dir", "read dir/a.md", "findings dir/a.md"]
        );
        assert_eq!(outcome.stopped, Some(ReportFault::Closed));
        assert_eq!(outcome.exit_code(), 2);
    }

    #[test]
    fn an_unwritable_reporter_reports_one_output_fault_and_stops() {
        let failure = ReportFault::Unwritable {
            detail: "No space left on device (os error 28)".to_string(),
        };
        let (outcome, calls) = Scenario::default()
            .source("a.md", "trail  \n")
            .source("b.md", "trail  \n")
            .fail_findings(1, failure.clone())
            .run(&["a.md", "b.md"]);

        assert_eq!(
            trace(&calls),
            [
                "resolve a.md",
                "read a.md",
                "findings a.md",
                "fault standard output: cannot be written (No space left on device (os error 28))",
            ]
        );
        assert_eq!(outcome.stopped, Some(failure));
        assert_eq!(outcome.faults, 0);
        assert_eq!(outcome.exit_code(), 2);
    }

    #[test]
    fn a_failure_at_finish_stops_the_run_with_exit_two() {
        let failure = ReportFault::Unwritable {
            detail: "Input/output error (os error 5)".to_string(),
        };
        let (outcome, calls) = Scenario::default()
            .source("clean.md", "clean\n")
            .fail_finish(failure.clone())
            .run(&["clean.md"]);

        assert_eq!(
            trace(&calls),
            [
                "resolve clean.md",
                "read clean.md",
                "finish",
                "fault standard output: cannot be written (Input/output error (os error 5))",
            ]
        );
        assert_eq!(outcome.stopped, Some(failure));
        assert_eq!(outcome.exit_code(), 2);

        // A closed output at the end is not reported, but still exits 2.
        let (outcome, calls) = Scenario::default()
            .source("clean.md", "clean\n")
            .fail_finish(ReportFault::Closed)
            .run(&["clean.md"]);

        assert_eq!(
            trace(&calls),
            ["resolve clean.md", "read clean.md", "finish"]
        );
        assert_eq!(outcome.stopped, Some(ReportFault::Closed));
        assert_eq!(outcome.exit_code(), 2);
    }
}
