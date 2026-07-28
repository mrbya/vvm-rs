//! End-to-end coverage command execution and exit-status policy.

use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vvm_core::{
    CoverageArtifact, CoverageBinDetail, CoverageMerge, CoverageMergePolicy, CoverageReport,
    CoverageReportOptions,
};

use crate::cli::{Cli, CliBinDetail, CliMergePolicy, Command, CoverageCommand};
use crate::command::{CargoInvocation, TestCommandKind, cargo_executable, parse};
use crate::error::CoverageCommandError;
use crate::metadata::query;
use crate::output::{CoverageOutputLayout, write_atomic_new};
use crate::process::{ProcessRunner, StdProcessRunner, TestProcessRequest};

/// Executes the parsed top-level command.
#[must_use]
pub fn run(cli: Cli) -> ExitCode {
    let Command::Coverage(command) = cli.command;
    let mut runner = StdProcessRunner;
    execute(command, &mut runner)
}

/// Executes coverage while keeping child and post-processing outcomes separate.
fn execute(command: CoverageCommand, runner: &mut impl ProcessRunner) -> ExitCode {
    let invocation = match parse(command.child, cargo_executable()) {
        Ok(invocation) => invocation,
        Err(error) => return report_error(&error),
    };

    let metadata = match query(runner, &invocation) {
        Ok(metadata) => metadata,
        Err(error) => return report_error(&error),
    };

    let layout = match CoverageOutputLayout::create(command.output, &command.name, &metadata) {
        Ok(layout) => layout,
        Err(error) => return report_error(&error),
    };

    eprintln!("VVM coverage output: {}", layout.root.display());

    let child = match run_child(runner, &invocation, &layout) {
        Ok(status) => status,
        Err(error) => return report_error(&error),
    };

    if !child.success() {
        eprintln!("VVM coverage child failed: {child}");
    }

    let postprocess = postprocess(
        &layout,
        command.merge_policy,
        command.bin_detail,
        command.no_inputs,
        command.fingerprints,
    );

    match postprocess {
        Ok(text) => {
            eprintln!("VVM coverage artifacts: {}", layout.artifacts.display());
            eprintln!("VVM merged coverage: {}", layout.merged.display());
            eprintln!("VVM text report: {}", layout.text.display());
            eprintln!("VVM HTML report: {}", layout.html.display());

            if let Err(source) = std::io::stdout().write_all(text.as_bytes()) {
                return select_status(
                    child,
                    false,
                    Some(CoverageCommandError::WriteStdout { source }),
                );
            }

            select_status(child, true, None)
        }

        Err(error) => select_status(child, false, Some(error)),
    }
}

/// Runs the live Cargo child with the exact forwarded argument vector.
fn run_child(
    runner: &mut impl ProcessRunner,
    invocation: &CargoInvocation,
    layout: &CoverageOutputLayout,
) -> Result<std::process::ExitStatus, CoverageCommandError> {
    let current_dir =
        std::env::current_dir().map_err(|source| CoverageCommandError::SpawnChild {
            command: invocation.arguments.clone(),
            source,
        })?;

    let request = TestProcessRequest {
        program: invocation.cargo.clone(),
        toolchain: invocation.toolchain.clone(),
        arguments: invocation.arguments.clone(),
        current_dir,
        coverage_dir: layout.artifacts.clone(),
        nextest_retries: matches!(invocation.kind, TestCommandKind::NextestRun),
    };

    runner
        .run_tests(&request)
        .map_err(|source| CoverageCommandError::SpawnChild {
            command: child_command(invocation),
            source,
        })
}

/// Performs generic non-recursive discovery and core-owned merge/rendering.
fn postprocess(
    layout: &CoverageOutputLayout,
    merge_policy: CliMergePolicy,
    bin_detail: CliBinDetail,
    no_inputs: bool,
    fingerprints: bool,
) -> Result<String, CoverageCommandError> {
    let artifacts = discover_artifacts(&layout.artifacts)?;
    let merge = CoverageMerge::from_files(map_policy(merge_policy), artifacts)?;

    merge.write_to(&layout.merged)?;

    let options = CoverageReportOptions::default()
        .with_bin_detail(map_detail(bin_detail))
        .with_inputs(!no_inputs)
        .with_fingerprints(fingerprints);
    let report = CoverageReport::with_options(&merge, options);
    let text = report.to_text();
    let html = report.to_html();

    write_atomic_new(&layout.text, text.as_bytes())?;
    write_atomic_new(&layout.html, html.as_bytes())?;

    // Return the exact content written to the text report and stdout.
    Ok(text)
}

/// Discovers sorted per-test documents owned by this run only.
fn discover_artifacts(directory: &Path) -> Result<Vec<PathBuf>, CoverageCommandError> {
    let entries =
        fs::read_dir(directory).map_err(|source| CoverageCommandError::ReadArtifactDirectory {
            path: directory.to_owned(),
            source,
        })?;

    let mut artifacts = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| CoverageCommandError::ReadArtifactEntry {
            path: directory.to_owned(),
            source,
        })?;

        let path = entry.path();

        let metadata =
            entry
                .metadata()
                .map_err(|source| CoverageCommandError::ReadArtifactEntry {
                    path: directory.to_owned(),
                    source,
                })?;

        let selected = metadata.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(CoverageArtifact::FILE_SUFFIX));

        if selected {
            artifacts.push(path);
        }
    }

    artifacts.sort_unstable();

    if artifacts.is_empty() {
        return Err(CoverageCommandError::NoArtifacts {
            path: directory.to_owned(),
        });
    }

    Ok(artifacts)
}

/// Converts stable CLI spelling to the core policy.
const fn map_policy(policy: CliMergePolicy) -> CoverageMergePolicy {
    match policy {
        CliMergePolicy::PassedOnly => CoverageMergePolicy::passed_only(),
        CliMergePolicy::PassedAndFailed => CoverageMergePolicy::passed_and_failed(),
        CliMergePolicy::All => CoverageMergePolicy::all(),
    }
}

/// Converts stable CLI spelling to the core report option.
const fn map_detail(detail: CliBinDetail) -> CoverageBinDetail {
    match detail {
        CliBinDetail::None => CoverageBinDetail::None,
        CliBinDetail::Uncovered => CoverageBinDetail::Uncovered,
        CliBinDetail::All => CoverageBinDetail::All,
    }
}

/// Builds a diagnostic representation of the child command.
fn child_command(invocation: &CargoInvocation) -> Vec<OsString> {
    let mut command = vec![invocation.cargo.clone()];

    command.extend(invocation.arguments.clone());
    command
}

/// Applies the documented primary-child status policy.
fn select_status(
    status: std::process::ExitStatus,
    postprocessed: bool,
    error: Option<CoverageCommandError>,
) -> ExitCode {
    if let Some(error) = error {
        eprintln!("VVM coverage post-processing failed: {error}");
    }

    if status.success() {
        return if postprocessed {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(2)
        };
    }

    child_exit_code(status)
}

/// Returns the portable child code or the documented fallback for signals.
fn child_exit_code(status: std::process::ExitStatus) -> ExitCode {
    status.code().map_or_else(
        || ExitCode::from(1),
        |code| u8::try_from(code).map_or_else(|_| ExitCode::from(1), ExitCode::from),
    )
}

/// Emits a pre-child orchestration failure.
fn report_error(error: &CoverageCommandError) -> ExitCode {
    eprintln!("VVM coverage failed: {error}");
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::process::{Command, ExitStatus};
    use std::{fs, io};

    use tempfile::tempdir;
    use vvm_core::CoverageArtifact;

    use super::{child_command, discover_artifacts, execute, run_child, select_status};
    use crate::cli::{CliBinDetail, CliMergePolicy, CoverageCommand};
    use crate::command::parse;
    use crate::error::CoverageCommandError;
    use crate::process::{MetadataRequest, ProcessOutput, ProcessRunner, TestProcessRequest};

    struct FakeRunner {
        metadata: ProcessOutput,
        child_status: ExitStatus,
        test_request: Option<TestProcessRequest>,
    }

    impl ProcessRunner for FakeRunner {
        fn cargo_metadata(&mut self, _: &MetadataRequest) -> Result<ProcessOutput, io::Error> {
            Ok(ProcessOutput {
                status: self.metadata.status,
                stdout: self.metadata.stdout.clone(),
                stderr: self.metadata.stderr.clone(),
            })
        }

        fn run_tests(&mut self, request: &TestProcessRequest) -> Result<ExitStatus, io::Error> {
            self.test_request = Some(request.clone());

            Ok(self.child_status)
        }
    }

    fn status(program: &str) -> Result<ExitStatus, io::Error> {
        Command::new(program).status()
    }

    fn command(output: PathBuf) -> CoverageCommand {
        CoverageCommand {
            output: Some(output),
            name: String::from("coverage"),
            merge_policy: CliMergePolicy::PassedOnly,
            bin_detail: CliBinDetail::Uncovered,
            no_inputs: false,
            fingerprints: false,
            child: ["nextest", "run", "--workspace"]
                .map(OsString::from)
                .to_vec(),
        }
    }

    #[test]
    fn rejects_empty_artifact_directory() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;

        assert!(discover_artifacts(directory.path()).is_err());

        Ok(())
    }

    #[test]
    fn ignores_unrelated_documents() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;

        fs::write(directory.path().join("ignored.json"), "{}")?;
        fs::write(directory.path().join("old.vvmcov-merged.json"), "{}")?;
        fs::write(
            directory
                .path()
                .join(format!("test{}", CoverageArtifact::FILE_SUFFIX)),
            "{}",
        )?;

        assert_eq!(discover_artifacts(directory.path())?.len(), 1);

        Ok(())
    }

    #[test]
    fn discovers_artifacts_in_path_order() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let first = directory.path().join("a.vvmcov.json");
        let second = directory.path().join("z.vvmcov.json");
        fs::write(&second, "{}")?;
        fs::write(&first, "{}")?;
        fs::create_dir_all(directory.path().join("nested.vvmcov.json"))?;

        assert_eq!(discover_artifacts(directory.path())?, [first, second]);

        Ok(())
    }

    #[test]
    fn run_child_preserves_forwarded_arguments_and_nextest_policy()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let invocation = parse(
            ["+nightly", "nextest", "run", "--workspace"]
                .map(OsString::from)
                .to_vec(),
            OsString::from("selected-cargo"),
        )?;
        let layout = crate::output::CoverageOutputLayout::create(
            Some(directory.path().join("coverage")),
            "coverage",
            &crate::metadata::WorkspaceMetadata {
                target_directory: directory.path().join("target"),
            },
        )?;
        let mut runner = FakeRunner {
            metadata: ProcessOutput {
                status: status("true")?,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
            child_status: status("true")?,
            test_request: None,
        };

        run_child(&mut runner, &invocation, &layout)?;

        let request = runner
            .test_request
            .ok_or("test child request was not captured")?;
        assert_eq!(request.program, "selected-cargo");
        assert_eq!(request.toolchain, Some(OsString::from("+nightly")));
        assert_eq!(request.arguments, invocation.arguments);
        assert_eq!(request.coverage_dir, layout.artifacts);
        assert!(request.nextest_retries);

        Ok(())
    }

    #[test]
    fn preserves_failed_child_status_when_postprocessing_fails()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let mut runner = FakeRunner {
            metadata: ProcessOutput {
                status: status("true")?,
                stdout: br#"{"workspace_root":"/workspace","target_directory":"/target"}"#.to_vec(),
                stderr: Vec::new(),
            },
            child_status: status("false")?,
            test_request: None,
        };

        let result = execute(command(directory.path().join("coverage")), &mut runner);

        assert_eq!(result, std::process::ExitCode::from(1));
        assert!(runner.test_request.is_some());

        Ok(())
    }

    #[test]
    fn reports_postprocessing_failure_after_successful_child()
    -> Result<(), Box<dyn std::error::Error>> {
        let error = CoverageCommandError::NoArtifacts {
            path: PathBuf::from("artifacts"),
        };

        assert_eq!(
            select_status(status("true")?, false, Some(error)),
            std::process::ExitCode::from(2)
        );

        Ok(())
    }

    #[test]
    fn builds_child_diagnostic_with_toolchain() -> Result<(), Box<dyn std::error::Error>> {
        let invocation = parse(
            ["+nightly", "test", "--workspace"]
                .map(OsString::from)
                .to_vec(),
            OsString::from("selected-cargo"),
        )?;

        assert_eq!(
            child_command(&invocation),
            ["selected-cargo", "+nightly", "test", "--workspace"].map(OsString::from)
        );

        Ok(())
    }
}
