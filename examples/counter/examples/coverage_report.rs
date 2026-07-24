//! Offline reporting for the counter functional-coverage example.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use thiserror::Error;
use vvm::{
    CoverageArtifact, CoverageMerge, CoverageMergeError, CoverageMergePolicy,
    CoveragePersistenceError, CoverageReport,
};

/// Counter coverage-report postprocessor failure.
#[derive(Debug, Error)]
enum Error {
    /// The artifact-directory argument was omitted.
    #[error("missing artifact-directory argument")]
    MissingArtifactDirectory,
    /// The output-directory argument was omitted.
    #[error("missing output-directory argument")]
    MissingOutputDirectory,
    /// An unexpected positional argument was supplied.
    #[error("unexpected extra argument `{argument}`")]
    ExtraArgument {
        /// Extra argument text.
        argument: String,
    },
    /// Artifact-directory enumeration failed.
    #[error("failed to read artifact directory `{path}`: {source}")]
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// Reading one artifact-directory entry failed.
    #[error("failed to read an artifact directory entry: {source}")]
    ReadDirectoryEntry {
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// No per-test coverage artifacts were selected.
    #[error("no per-test coverage artifacts found in `{path}`")]
    NoArtifacts {
        /// Searched directory.
        path: PathBuf,
    },
    /// Offline coverage merge failed.
    #[error(transparent)]
    Merge(#[from] CoverageMergeError),
    /// Persisting the merged document failed.
    #[error(transparent)]
    Persistence(#[from] CoveragePersistenceError),
    /// Creating an output directory failed.
    #[error("failed to create output directory `{path}`: {source}")]
    CreateOutputDirectory {
        /// Output directory.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// Writing a rendered report failed.
    #[error("failed to write report `{path}`: {source}")]
    WriteReport {
        /// Report path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// Writing the text report to stdout failed.
    #[error("failed to write report to stdout: {source}")]
    WriteStdout {
        /// Underlying output error.
        #[source]
        source: std::io::Error,
    },
}

/// Runs the counter-specific postprocessor.
fn main() -> Result<(), Error> {
    let (artifact_directory, output_directory) = parse_arguments(env::args().skip(1))?;
    let text = generate_reports(&artifact_directory, &output_directory)?;

    std::io::stdout()
        .write_all(text.as_bytes())
        .map_err(|source| Error::WriteStdout { source })?;

    Ok(())
}

/// Parses the two required positional directory arguments.
fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<(PathBuf, PathBuf), Error> {
    let artifact_directory = arguments.next().ok_or(Error::MissingArtifactDirectory)?;
    let output_directory = arguments.next().ok_or(Error::MissingOutputDirectory)?;

    if let Some(argument) = arguments.next() {
        return Err(Error::ExtraArgument { argument });
    }

    Ok((
        PathBuf::from(artifact_directory),
        PathBuf::from(output_directory),
    ))
}

/// Discovers sorted per-test artifacts in one non-recursive directory.
fn discover_artifacts(directory: &Path) -> Result<Vec<PathBuf>, Error> {
    let entries = fs::read_dir(directory).map_err(|source| Error::ReadDirectory {
        path: directory.to_owned(),
        source,
    })?;
    let mut artifacts = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| Error::ReadDirectoryEntry { source })?;
        let path = entry.path();
        let is_regular_file = entry
            .metadata()
            .map_err(|source| Error::ReadDirectoryEntry { source })?
            .is_file();
        let is_artifact = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(CoverageArtifact::FILE_SUFFIX));

        if is_regular_file && is_artifact {
            artifacts.push(path);
        }
    }

    artifacts.sort_unstable();

    if artifacts.is_empty() {
        return Err(Error::NoArtifacts {
            path: directory.to_owned(),
        });
    }

    Ok(artifacts)
}

/// Merges discovered artifacts and writes deterministic counter reports.
fn generate_reports(artifact_directory: &Path, output_directory: &Path) -> Result<String, Error> {
    let artifacts = discover_artifacts(artifact_directory)?;
    let merge = CoverageMerge::from_files(CoverageMergePolicy::passed_only(), artifacts)?;

    fs::create_dir_all(output_directory).map_err(|source| Error::CreateOutputDirectory {
        path: output_directory.to_owned(),
        source,
    })?;

    let merged_path = output_directory.join(format!("counter{}", CoverageMerge::FILE_SUFFIX));
    merge.write_to(merged_path)?;

    let report = CoverageReport::new(&merge);
    let text = report.to_text();
    let html = report.to_html();
    write_new_file(
        &output_directory.join(format!("counter{}", CoverageReport::TEXT_FILE_SUFFIX)),
        &text,
    )?;
    write_new_file(
        &output_directory.join(format!("counter{}", CoverageReport::HTML_FILE_SUFFIX)),
        &html,
    )?;

    Ok(text)
}

/// Writes one report without replacing a pre-existing result.
fn write_new_file(path: &Path, contents: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| Error::WriteReport {
            path: path.to_owned(),
            source,
        })?;

    file.write_all(contents.as_bytes())
        .map_err(|source| Error::WriteReport {
            path: path.to_owned(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use vvm::{
        Bin, CoverageArtifact, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor,
        CoverageItemRef, CoverageMerge, CoverageReport, CoverageSession, Coverpoint, TestStatus,
    };

    use super::{discover_artifacts, generate_reports, parse_arguments};

    /// Minimal valid coverage group for postprocessor fixtures.
    struct Group {
        /// Group identity.
        instance: CoverageGroupInstance,
        /// One covered item.
        point: Coverpoint<u8>,
    }

    impl CoverageGroup for Group {
        fn instance(&self) -> &CoverageGroupInstance {
            &self.instance
        }

        fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
            visitor.visit(CoverageItemRef::coverpoint(&self.point));
        }
    }

    /// Writes one valid input artifact.
    fn artifact(directory: &std::path::Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut point = Coverpoint::builder("operation")
            .bin(Bin::value("count", 1_u8))
            .build()?;
        point.sample(&1)?;
        let group = Group {
            instance: CoverageGroupInstance::new_with_revision(
                "counter_coverage",
                "dut.counter",
                1,
            )?,
            point,
        };
        let mut session = CoverageSession::new(name)?;
        session.capture(&group)?;
        let snapshot = session.finish().ok_or("missing session snapshot")?;
        let artifact = CoverageArtifact::from_session(TestStatus::Passed, None, snapshot)?;
        artifact.write_to(directory.join(format!("{name}{}", CoverageArtifact::FILE_SUFFIX)))?;
        Ok(())
    }

    #[test]
    fn discovers_and_generates_reports() -> Result<(), Box<dyn std::error::Error>> {
        let artifacts = tempfile::tempdir()?;
        let output = tempfile::tempdir()?;
        artifact(artifacts.path(), "b")?;
        artifact(artifacts.path(), "a")?;
        std::fs::write(artifacts.path().join("ignored.json"), "{}")?;
        std::fs::create_dir(artifacts.path().join("directory.vvmcov.json"))?;
        std::fs::write(artifacts.path().join("old.vvmcov-merged.json"), "{}")?;

        let discovered = discover_artifacts(artifacts.path())?;
        assert_eq!(discovered.len(), 2);
        assert!(discovered[0] < discovered[1]);

        let text = generate_reports(artifacts.path(), output.path())?;
        let merged = output.path().join("counter.vvmcov-merged.json");
        let text_path = output.path().join("counter.vvmcov.txt");
        let html_path = output.path().join("counter.vvmcov.html");
        let parsed = CoverageMerge::read_from(&merged)?;
        let report = CoverageReport::new(&parsed);
        let html = std::fs::read_to_string(&html_path)?;

        assert_eq!(std::fs::read_to_string(&text_path)?, text);
        assert_eq!(text, report.to_text());
        assert!(text.ends_with(&format!("{}\n", report.gitlab_metric())));
        assert_eq!(html, report.to_html());
        assert!(html.starts_with("<!doctype html>"));
        assert!(!html.contains("<script"));

        Ok(())
    }

    #[test]
    fn rejects_missing_arguments_and_empty_directories() -> Result<(), Box<dyn std::error::Error>> {
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(parse_arguments(["artifacts".to_owned()].into_iter()).is_err());
        assert!(
            parse_arguments(
                [
                    "artifacts".to_owned(),
                    "output".to_owned(),
                    "extra".to_owned()
                ]
                .into_iter()
            )
            .is_err()
        );
        let directory = tempfile::tempdir()?;
        assert!(discover_artifacts(directory.path()).is_err());
        Ok(())
    }
}
