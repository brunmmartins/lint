use std::path::{Path, PathBuf};

use lint_domain::{Finding, RULES};

use crate::{LintFault, MarkdownParser, SourceReader, Walker};

/// The result of linting every input path given to [`run_lint`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintOutcome {
    /// Every finding, paired with the file it came from, in output order (input order,
    /// then each directory's sorted walk order, then `(line, column)` within one file).
    pub findings: Vec<(PathBuf, Finding)>,
    /// Every fault encountered, in the input order that produced it. A fault on one path never
    /// stops the other paths from being processed.
    pub faults: Vec<LintFault>,
}

/// Walks, reads, parses, and checks every path in `inputs`, in order.
///
/// A fault on one input (not found, unreadable, too large, invalid UTF-8) is recorded and that
/// input is skipped; every other input is still processed, so one bad path never
/// hides the problems in the others.
pub fn run_lint<W, R, P>(inputs: &[PathBuf], walker: &W, reader: &R, parser: &P) -> LintOutcome
where
    W: Walker,
    R: SourceReader,
    P: MarkdownParser,
{
    let mut outcome = LintOutcome::default();

    for input in inputs {
        match walker.resolve(input) {
            Ok(files) => {
                for file in files {
                    lint_one_file(&file, reader, parser, &mut outcome);
                }
            }
            Err(fault) => outcome.faults.push(LintFault::from(fault)),
        }
    }

    outcome
}

fn lint_one_file<R, P>(file: &Path, reader: &R, parser: &P, outcome: &mut LintOutcome)
where
    R: SourceReader,
    P: MarkdownParser,
{
    let source = match reader.read(file) {
        Ok(source) => source,
        Err(fault) => {
            outcome.faults.push(LintFault::from(fault));
            return;
        }
    };

    let document = parser.parse(&source);
    let mut findings: Vec<Finding> = RULES
        .iter()
        .flat_map(|rule| rule.check(&document))
        .collect();
    findings.sort_by_key(Finding::location);

    outcome.findings.extend(
        findings
            .into_iter()
            .map(|finding| (file.to_path_buf(), finding)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReadFault, WalkFault};
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct FakeWalker {
        resolutions: HashMap<PathBuf, Result<Vec<PathBuf>, WalkFault>>,
    }

    impl Walker for FakeWalker {
        fn resolve(&self, input: &Path) -> Result<Vec<PathBuf>, WalkFault> {
            self.resolutions
                .get(input)
                .cloned()
                .unwrap_or_else(|| Ok(vec![input.to_path_buf()]))
        }
    }

    struct FakeReader {
        sources: HashMap<PathBuf, Result<String, ReadFault>>,
    }

    impl SourceReader for FakeReader {
        fn read(&self, path: &Path) -> Result<String, ReadFault> {
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

    /// Records every path it was asked to resolve/read, so ordering assertions can rely on it.
    struct RecordingParser {
        calls: RefCell<Vec<String>>,
    }

    impl MarkdownParser for RecordingParser {
        fn parse(&self, source: &str) -> lint_domain::Document {
            self.calls.borrow_mut().push(source.to_string());
            lint_domain::Document::from_source(source)
        }
    }

    #[test]
    fn clean_file_produces_no_findings_and_no_faults() {
        let walker = FakeWalker {
            resolutions: HashMap::new(),
        };
        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("clean.md"), Ok("clean\n".to_string()));
        let reader = FakeReader { sources };

        let outcome = run_lint(&[PathBuf::from("clean.md")], &walker, &reader, &FakeParser);

        assert!(outcome.findings.is_empty());
        assert!(outcome.faults.is_empty());
    }

    #[test]
    fn a_fault_on_one_input_does_not_stop_the_others() {
        let mut resolutions = HashMap::new();
        resolutions.insert(
            PathBuf::from("missing.md"),
            Err(WalkFault::NotFound {
                path: PathBuf::from("missing.md"),
            }),
        );
        let walker = FakeWalker { resolutions };

        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("dirty.md"), Ok("trail  \n".to_string()));
        let reader = FakeReader { sources };

        let inputs = vec![PathBuf::from("missing.md"), PathBuf::from("dirty.md")];
        let outcome = run_lint(&inputs, &walker, &reader, &FakeParser);

        assert_eq!(outcome.faults.len(), 1);
        assert_eq!(outcome.findings.len(), 1);
        assert_eq!(outcome.findings[0].0, PathBuf::from("dirty.md"));
    }

    #[test]
    fn findings_preserve_input_order_then_walk_order() {
        let mut resolutions = HashMap::new();
        resolutions.insert(
            PathBuf::from("dir"),
            Ok(vec![PathBuf::from("dir/a.md"), PathBuf::from("dir/b.md")]),
        );
        let walker = FakeWalker { resolutions };

        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("dir/a.md"), Ok("trail  \n".to_string()));
        sources.insert(PathBuf::from("dir/b.md"), Ok("trail  \n".to_string()));
        let reader = FakeReader { sources };

        let outcome = run_lint(&[PathBuf::from("dir")], &walker, &reader, &FakeParser);

        assert_eq!(outcome.findings.len(), 2);
        assert_eq!(outcome.findings[0].0, PathBuf::from("dir/a.md"));
        assert_eq!(outcome.findings[1].0, PathBuf::from("dir/b.md"));
    }

    #[test]
    fn findings_within_one_file_are_sorted_by_line_and_column() {
        let walker = FakeWalker {
            resolutions: HashMap::new(),
        };
        // Missing trailing newline (reported at the last line) and a trailing-space line earlier
        // in the file: independent locations, MD009 before MD047 by position.
        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("both.md"), Ok("trail  \nclean".to_string()));
        let reader = FakeReader { sources };

        let outcome = run_lint(&[PathBuf::from("both.md")], &walker, &reader, &FakeParser);

        assert_eq!(outcome.findings.len(), 2);
        assert!(outcome.findings[0].1.location() < outcome.findings[1].1.location());
    }

    #[test]
    fn every_resolved_file_is_parsed_exactly_once() {
        let mut resolutions = HashMap::new();
        resolutions.insert(
            PathBuf::from("dir"),
            Ok(vec![PathBuf::from("dir/a.md"), PathBuf::from("dir/b.md")]),
        );
        let walker = FakeWalker { resolutions };

        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("dir/a.md"), Ok("a\n".to_string()));
        sources.insert(PathBuf::from("dir/b.md"), Ok("b\n".to_string()));
        let reader = FakeReader { sources };

        let parser = RecordingParser {
            calls: RefCell::new(Vec::new()),
        };
        run_lint(&[PathBuf::from("dir")], &walker, &reader, &parser);

        assert_eq!(
            parser.calls.into_inner(),
            vec!["a\n".to_string(), "b\n".to_string()]
        );
    }
}
