use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};
use std::{env, fmt, fs};

use thiserror::Error;
use vvm_core::{
    CoverageArtifact, CoveragePersistenceError, ParseReplayTokenError, ParseSeedError, ReplayToken,
    Seed, TestDescriptor, TestRegistryError, TestRun, TestRunConfig,
};

/// Environment variable selecting a seed-derived replay token.
const ENV_SEED: &str = "VVM_SEED";
/// Environment variable selecting an explicit replay token.
const ENV_REPLAY: &str = "VVM_REPLAY";
/// Environment variable overriding replayable test cycle counts.
const ENV_CYCLES: &str = "VVM_CYCLES";
/// Environment variable selecting the root directory for trace files.
const ENV_TRACE_DIR: &str = "VVM_TRACE_DIR";
/// Environment variable selecting the root directory for coverage artifacts.
const ENV_COVERAGE_DIR: &str = "VVM_COVERAGE_DIR";

/// Runs one VVM descriptor through the standard Rust test harness bridge.
///
/// # Errors
///
/// Returns a formatted test failure when configuration or execution fails.
pub fn run_test(descriptor: &TestDescriptor) -> Result<(), TestFailure> {
    let overrides = EnvOverrides::parse(descriptor.name())?;

    run_test_with_overrides(descriptor, &overrides)
}

/// Runs one descriptor with already-parsed environment overrides.
fn run_test_with_overrides(
    descriptor: &TestDescriptor,
    overrides: &EnvOverrides,
) -> Result<(), TestFailure> {
    let config = overrides
        .config_for(descriptor)
        .map_err(|error| TestFailure::configuration(descriptor.name(), error))?;
    let run = descriptor.run(&config).map_err(|source| {
        TestFailure::configuration(descriptor.name(), HarnessError::Registry { source })
    })?;

    let test_failure = (!run.passed()).then(|| TestFailure::from_run(&run));
    let test_name = run.test().name();
    let (outcome, coverage) = run.into_parts();

    let persistence_result = coverage
        .map(|session| {
            let artifact =
                CoverageArtifact::from_session(outcome.status(), outcome.replay_token(), session)
                    .map_err(|source| {
                    TestFailure::configuration(
                        test_name,
                        HarnessError::CoveragePersistence { source },
                    )
                })?;
            let path = overrides.coverage_path_for(test_name)?;

            artifact.write_to(path).map_err(|source| {
                TestFailure::configuration(test_name, HarnessError::CoveragePersistence { source })
            })
        })
        .transpose();

    match (test_failure, persistence_result) {
        (Some(failure), Ok(_)) => Err(failure),
        (Some(failure), Err(persistence)) => {
            Err(failure.with_coverage_persistence_error(persistence))
        }
        (None, Ok(_)) => Ok(()),
        (None, Err(persistence)) => Err(persistence),
    }
}

/// Human-readable failure returned from generated standard Rust tests.
pub struct TestFailure {
    /// Fully formatted failure report.
    message: String,
}

impl TestFailure {
    /// Creates a configuration or setup failure.
    fn configuration(name: &str, details: impl std::fmt::Display) -> Self {
        Self {
            message: format!("VVM test `{name}` failed\n\n{details}"),
        }
    }

    /// Creates a failure from one executed descriptor run.
    fn from_run(run: &TestRun<'_>) -> Self {
        let outcome = run.outcome();
        let mut message = format!("VVM test `{}` failed", run.test().name());

        if let Some(replay) = outcome.replay_token() {
            message.push_str("\n\nReplay:\n  ");
            message.push_str(&replay.to_string());
        }

        if let Some(statistics) = outcome.statistics() {
            if write!(
                message,
                "\n\nStatistics:\n  cycles: {}\n  checks: {}\n  failures: {}",
                statistics.cycles(),
                statistics.checks(),
                statistics.check_failures(),
            )
            .is_err()
            {}
        }

        message.push_str("\n\nReport:\n");
        message.push_str(outcome.report());

        Self { message }
    }

    /// Appends a coverage persistence failure to an existing test failure.
    fn with_coverage_persistence_error(mut self, error: impl fmt::Display) -> Self {
        self.message.push_str("\n\nCoverage persistence error:\n  ");
        self.message.push_str(&error.to_string());
        self
    }
}

// The standard test harness only retains this final rendered report.
impl fmt::Debug for TestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

// Display intentionally matches Debug so test failures preserve the full report.
impl fmt::Display for TestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

// No source is exposed because this is the terminal test-harness report wrapper.
impl std::error::Error for TestFailure {}

/// Typed setup failure retained until the final test-harness reporting boundary.
#[derive(Debug, Error)]
enum HarnessError {
    /// Replay and seed overrides conflict.
    #[error("environment variables `{first}` and `{second}` are mutually exclusive")]
    ConflictingEnvironment {
        /// First conflicting variable.
        first: &'static str,
        /// Second conflicting variable.
        second: &'static str,
    },
    /// A replay override was malformed.
    #[error("invalid `{name}` value `{value}`: {source}")]
    InvalidReplay {
        /// Environment variable name.
        name: &'static str,
        /// Original value.
        value: String,
        /// Typed parse source.
        #[source]
        source: ParseReplayTokenError,
    },
    /// A seed override was malformed.
    #[error("invalid `{name}` value `{value}`: {source}")]
    InvalidSeed {
        /// Environment variable name.
        name: &'static str,
        /// Original value.
        value: String,
        /// Typed parse source.
        #[source]
        source: ParseSeedError,
    },
    /// Test registration or configuration failed.
    #[error("test configuration failed: {source}")]
    Registry {
        /// Typed registry source.
        #[source]
        source: TestRegistryError,
    },
    /// Coverage artifact construction or persistence failed.
    #[error("coverage persistence failed: {source}")]
    CoveragePersistence {
        /// Typed persistence source.
        #[source]
        source: CoveragePersistenceError,
    },
}

/// Parsed environment overrides shared across all standard test wrappers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EnvOverrides {
    /// Global replay override, if any.
    replay_token: Option<ReplayToken>,

    /// Global cycle-count override, if any.
    cycles: Option<u64>,

    /// Root directory for generated waveform files, if explicitly configured.
    trace_root: Option<PathBuf>,

    /// Root directory for generated coverage artifacts, if explicitly configured.
    coverage_root: Option<PathBuf>,
}

impl EnvOverrides {
    /// Parses process environment overrides.
    fn parse(test_name: &str) -> Result<Self, TestFailure> {
        Self::parse_with(test_name, |name| env::var(name).ok())
    }

    /// Parses environment overrides through an injected lookup function.
    fn parse_with(
        test_name: &str,
        mut lookup: impl FnMut(&str) -> Option<String>,
    ) -> Result<Self, TestFailure> {
        let seed = lookup(ENV_SEED);
        let replay = lookup(ENV_REPLAY);

        if seed.is_some() && replay.is_some() {
            return Err(TestFailure::configuration(
                test_name,
                HarnessError::ConflictingEnvironment {
                    first: ENV_SEED,
                    second: ENV_REPLAY,
                },
            ));
        }

        let replay_token = if let Some(value) = replay {
            Some(value.parse::<ReplayToken>().map_err(|source| {
                TestFailure::configuration(
                    test_name,
                    HarnessError::InvalidReplay {
                        name: ENV_REPLAY,
                        value,
                        source,
                    },
                )
            })?)
        } else if let Some(value) = seed {
            let parsed_seed = value.parse::<Seed>().map_err(|source| {
                TestFailure::configuration(
                    test_name,
                    HarnessError::InvalidSeed {
                        name: ENV_SEED,
                        value,
                        source,
                    },
                )
            })?;
            Some(ReplayToken::new(parsed_seed))
        } else {
            None
        };

        let cycles = lookup(ENV_CYCLES)
            .map(|value| {
                value.parse::<u64>().map_err(|error| {
                    TestFailure::configuration(
                        test_name,
                        format!("invalid `{ENV_CYCLES}` value `{value}`: {error}"),
                    )
                })
            })
            .transpose()?;

        let trace_root = lookup(ENV_TRACE_DIR)
            .map(|value| {
                if value.is_empty() {
                    return Err(TestFailure::configuration(
                        test_name,
                        format!("environment variable `{ENV_TRACE_DIR}` must not be empty"),
                    ));
                }

                Ok(PathBuf::from(value))
            })
            .transpose()?;

        let coverage_root = lookup(ENV_COVERAGE_DIR)
            .map(|value| {
                if value.is_empty() {
                    return Err(TestFailure::configuration(
                        test_name,
                        format!("environment variable `{ENV_COVERAGE_DIR}` must not be empty"),
                    ));
                }

                Ok(PathBuf::from(value))
            })
            .transpose()?;

        Ok(Self {
            replay_token,
            cycles,
            trace_root,
            coverage_root,
        })
    }

    /// Builds the per-test configuration for one descriptor.
    fn config_for(&self, descriptor: &TestDescriptor) -> Result<TestRunConfig, TestFailure> {
        let mut config = TestRunConfig::new();

        if descriptor.capabilities().replay()
            && let Some(replay_token) = self.replay_token
        {
            config = config.with_replay_token(replay_token);
        }

        if descriptor.capabilities().cycles()
            && let Some(cycles) = self.cycles
        {
            config = config.with_cycles(cycles);
        }

        if descriptor.capabilities().trace() {
            config = config.with_trace_path(self.trace_path_for(descriptor)?);
        }

        Ok(config)
    }

    /// Resolves and prepares the waveform output path for one descriptor.
    fn trace_path_for(&self, descriptor: &TestDescriptor) -> Result<PathBuf, TestFailure> {
        let root = self.trace_root.clone().unwrap_or_else(default_trace_root);

        ensure_trace_root(descriptor.name(), &root)?;

        Ok(root.join(format!("{}.vcd", descriptor.name())))
    }

    /// Resolves one isolated artifact path for a descriptor.
    fn coverage_path_for(&self, test_name: &str) -> Result<PathBuf, TestFailure> {
        let root = self
            .coverage_root
            .clone()
            .unwrap_or_else(default_coverage_root);

        if root.exists() && !root.is_dir() {
            return Err(TestFailure::configuration(
                test_name,
                format!(
                    "coverage output path `{}` from `{ENV_COVERAGE_DIR}` is not a directory",
                    root.display()
                ),
            ));
        }

        Ok(root.join(format!(
            "pid-{}-{test_name}{}",
            std::process::id(),
            CoverageArtifact::FILE_SUFFIX
        )))
    }
}

/// Returns the default trace root for this process.
fn default_trace_root() -> PathBuf {
    PathBuf::from("target").join("vvm-trace").join(run_id())
}

/// Returns the stable identifier for this process-wide test invocation.
fn run_id() -> &'static str {
    static RUN_ID: OnceLock<String> = OnceLock::new();

    RUN_ID.get_or_init(|| {
        env::var("NEXTEST_RUN_ID").unwrap_or_else(|_| {
            let elapsed = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos();
            format!("pid-{}-{elapsed:032x}", std::process::id())
        })
    })
}

/// Returns the default coverage root for this process.
fn default_coverage_root() -> PathBuf {
    PathBuf::from("target").join("vvm-coverage").join(run_id())
}

/// Ensures the trace output root exists and is a directory.
fn ensure_trace_root(test_name: &str, root: &Path) -> Result<(), TestFailure> {
    if root.exists() && !root.is_dir() {
        return Err(TestFailure::configuration(
            test_name,
            format!(
                "trace output path `{}` from `{ENV_TRACE_DIR}` is not a directory",
                root.display(),
            ),
        ));
    }

    if root.exists() {
        return Ok(());
    }

    fs::create_dir_all(root).map_err(|error| {
        TestFailure::configuration(
            test_name,
            format!(
                "failed to create trace directory `{}` for `{ENV_TRACE_DIR}`: {error}",
                root.display(),
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::path::Path;

    use vvm_core::{
        Bin, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor, CoverageItemRef,
        TestCapabilities, TestContext, TestOutcome,
    };

    use super::{
        ENV_COVERAGE_DIR, ENV_CYCLES, ENV_REPLAY, ENV_SEED, ENV_TRACE_DIR, EnvOverrides,
        TestFailure, default_coverage_root, default_trace_root, ensure_trace_root,
        run_test_with_overrides,
    };
    use crate::random::{ReplayToken, Seed};
    use crate::test::TestDescriptor;

    fn descriptor(capabilities: TestCapabilities) -> TestDescriptor {
        TestDescriptor::new(
            "counter-random",
            "Counter random",
            |_config| TestOutcome::error("not used"),
            capabilities,
        )
    }

    struct CapturedCoverage {
        instance: CoverageGroupInstance,
        value: vvm_core::Coverpoint<u8>,
    }

    impl CoverageGroup for CapturedCoverage {
        fn instance(&self) -> &CoverageGroupInstance {
            &self.instance
        }

        fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
            visitor.visit(CoverageItemRef::coverpoint(&self.value));
        }
    }

    fn captures_coverage(context: &mut TestContext) -> TestOutcome {
        let mut value = match vvm_core::Coverpoint::builder("value")
            .bin(Bin::value("one", 1_u8))
            .build()
        {
            Ok(value) => value,
            Err(error) => return TestOutcome::error(error),
        };
        if let Err(error) = value.sample(&1) {
            return TestOutcome::error(error);
        }

        let coverage = match CoverageGroupInstance::new("sample", "dut.sample") {
            Ok(instance) => CapturedCoverage { instance, value },
            Err(error) => return TestOutcome::error(error),
        };
        if let Err(error) = context.capture_coverage(&coverage) {
            return TestOutcome::error(error);
        }

        TestOutcome::error("intentional failure after coverage capture")
    }

    #[test]
    fn parses_empty_environment() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = EnvOverrides::parse_with("counter-random", |_| None)?;

        assert_eq!(parsed, EnvOverrides::default());
        Ok(())
    }

    #[test]
    fn parses_seed_override() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_SEED).then(|| String::from("0x1234"))
        })?;

        assert_eq!(
            parsed.replay_token,
            Some(ReplayToken::new(Seed::new(0x1234)))
        );
        Ok(())
    }

    #[test]
    fn parses_replay_override() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_REPLAY).then(|| String::from("chacha8-v1:0123456789abcdef"))
        })?;

        assert_eq!(
            parsed.replay_token,
            Some("chacha8-v1:0123456789abcdef".parse()?),
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_seed_override() {
        let error = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_SEED).then(|| String::from("not-a-seed"))
        });

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("invalid `VVM_SEED` value `not-a-seed`"))
        );
    }

    #[test]
    fn rejects_invalid_replay_override() {
        let error = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_REPLAY).then(|| String::from("not-a-replay-token"))
        });

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("invalid `VVM_REPLAY` value `not-a-replay-token`"))
        );
    }

    #[test]
    fn rejects_seed_replay_conflict() {
        let error = EnvOverrides::parse_with("counter-random", |name| match name {
            ENV_SEED => Some(String::from("0x1234")),
            ENV_REPLAY => Some(String::from("chacha8-v1:0123456789abcdef")),
            _ => None,
        });

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("mutually exclusive"))
        );
    }

    #[test]
    fn parses_cycle_override() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_CYCLES).then(|| String::from("100"))
        })?;

        assert_eq!(parsed.cycles, Some(100));
        Ok(())
    }

    #[test]
    fn parses_coverage_root_from_environment() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("custom-coverage");
        let parsed = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_COVERAGE_DIR).then(|| root.display().to_string())
        })?;

        assert_eq!(parsed.coverage_root.as_deref(), Some(root.as_path()));
        Ok(())
    }

    #[test]
    fn rejects_empty_coverage_root() {
        let error = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_COVERAGE_DIR).then(String::new)
        });

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains(ENV_COVERAGE_DIR))
        );
    }

    #[test]
    fn rejects_invalid_cycle_override() {
        let error = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_CYCLES).then(|| String::from("abc"))
        });

        assert!(matches!(error, Err(ref failure) if format!("{failure:?}").contains(ENV_CYCLES)));
    }

    #[test]
    fn ignores_irrelevant_overrides_for_deterministic_tests()
    -> Result<(), Box<dyn std::error::Error>> {
        let overrides = EnvOverrides::parse_with("counter-random", |name| match name {
            ENV_SEED => Some(String::from("0x1234")),
            ENV_CYCLES => Some(String::from("64")),
            _ => None,
        })?;
        let config = overrides.config_for(&descriptor(TestCapabilities::new()))?;

        assert!(config.replay_token().is_none());
        assert!(config.cycles().is_none());
        Ok(())
    }

    #[test]
    fn resolves_trace_root_from_environment() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("custom-traces");
        let overrides = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_TRACE_DIR).then(|| root.display().to_string())
        })?;
        let path = overrides.trace_path_for(&descriptor(TestCapabilities::new().with_trace()))?;

        assert_eq!(path, root.join("counter-random.vcd"));
        assert!(root.is_dir());
        Ok(())
    }

    #[test]
    fn rejects_empty_trace_root() {
        let error = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_TRACE_DIR).then(String::new)
        });

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("must not be empty"))
        );
    }

    #[test]
    fn creates_default_trace_root_under_target() {
        let root = default_trace_root();

        assert!(root.starts_with(Path::new("target").join("vvm-trace")));
    }

    #[test]
    fn rejects_trace_root_when_path_is_file() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("trace-root");
        let _file = File::create(&path)?;

        let error = ensure_trace_root("counter-random", &path);

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("not a directory"))
        );
        Ok(())
    }

    #[test]
    fn config_for_applies_trace_only_to_traceable_tests() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("traces");
        let overrides = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_TRACE_DIR).then(|| root.display().to_string())
        })?;

        let config = overrides.config_for(&descriptor(TestCapabilities::new()))?;

        assert!(config.trace_path().is_none());
        Ok(())
    }

    #[test]
    fn config_for_applies_supported_overrides() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("traces");
        let overrides = EnvOverrides::parse_with("counter-random", |name| match name {
            ENV_REPLAY => Some(String::from("chacha8-v1:0123456789abcdef")),
            ENV_CYCLES => Some(String::from("64")),
            ENV_TRACE_DIR => Some(root.display().to_string()),
            _ => None,
        })?;
        let capabilities = TestCapabilities::new()
            .with_replay()
            .with_cycles()
            .with_trace();
        let config = overrides.config_for(&descriptor(capabilities))?;

        assert_eq!(
            config.replay_token(),
            Some("chacha8-v1:0123456789abcdef".parse()?),
        );
        assert_eq!(config.cycles(), Some(64));
        assert_eq!(config.trace_path(), Some(&root.join("counter-random.vcd")));
        Ok(())
    }

    #[test]
    fn coverage_path_uses_configured_root_and_process_test_name()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("coverage");
        let overrides = EnvOverrides::parse_with("counter-random", |name| {
            (name == ENV_COVERAGE_DIR).then(|| root.display().to_string())
        })?;
        let path = overrides.coverage_path_for("counter-random")?;

        assert_eq!(path.parent(), Some(root.as_path()));
        assert!(path.file_name().is_some_and(|name| {
            name.to_string_lossy()
                .starts_with(&format!("pid-{}-counter-random", std::process::id()))
        }));
        assert!(path.to_string_lossy().ends_with(".vvmcov.json"));
        assert!(!root.exists());
        Ok(())
    }

    #[test]
    fn default_coverage_root_is_under_target() {
        let root = default_coverage_root();

        assert!(root.starts_with(Path::new("target").join("vvm-coverage")));
    }

    #[test]
    fn coverage_path_rejects_a_file_root() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("coverage");
        let _file = File::create(&root)?;
        let overrides = EnvOverrides {
            coverage_root: Some(root),
            ..EnvOverrides::default()
        };

        let error = overrides.coverage_path_for("counter-random");

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("not a directory"))
        );
        Ok(())
    }

    #[test]
    fn bridge_persists_captured_coverage_after_a_test_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("coverage");
        let descriptor = TestDescriptor::new_with_context(
            "covered-failure",
            "Captures coverage before failure",
            captures_coverage,
            TestCapabilities::new(),
        );
        let overrides = EnvOverrides {
            coverage_root: Some(root.clone()),
            ..EnvOverrides::default()
        };

        let result = run_test_with_overrides(&descriptor, &overrides);
        let artifact = root.join(format!(
            "pid-{}-covered-failure.vvmcov.json",
            std::process::id()
        ));

        assert!(result.is_err());
        assert!(artifact.is_file());
        assert_eq!(
            vvm_core::CoverageArtifact::read_from(artifact)?.test_name(),
            "covered-failure"
        );
        Ok(())
    }

    #[test]
    fn bridge_preserves_test_failure_when_coverage_persistence_fails()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("coverage");
        let _file = File::create(&root)?;
        let descriptor = TestDescriptor::new_with_context(
            "covered-failure",
            "Captures coverage before failure",
            captures_coverage,
            TestCapabilities::new(),
        );
        let overrides = EnvOverrides {
            coverage_root: Some(root),
            ..EnvOverrides::default()
        };

        let error = run_test_with_overrides(&descriptor, &overrides).expect_err("test must fail");
        let rendered = format!("{error:?}");

        assert!(rendered.contains("intentional failure after coverage capture"));
        assert!(rendered.contains("Coverage persistence error"));
        assert!(rendered.contains("not a directory"));
        Ok(())
    }

    #[test]
    fn bridge_reports_replay_metadata_and_execution_details() {
        let descriptor = TestDescriptor::new(
            "replay-failure",
            "Reports replay metadata",
            |_config| {
                TestOutcome::error("intentional replay failure")
                    .with_replay_token(ReplayToken::new(Seed::new(0x1234)))
            },
            TestCapabilities::new().with_replay(),
        );

        let error = run_test_with_overrides(&descriptor, &EnvOverrides::default())
            .expect_err("test must fail");
        let rendered = format!("{error}");

        assert!(rendered.contains("Replay:\n  chacha8-v1:0000000000001234"));
        assert!(rendered.contains("Report:\nTest execution error:"));
        assert!(rendered.contains("intentional replay failure"));
    }

    #[test]
    fn bridge_wraps_registry_configuration_errors() {
        let descriptor = TestDescriptor::new(
            "Invalid name",
            "Invalid registry name",
            |_config| TestOutcome::error("not reached"),
            TestCapabilities::new(),
        );

        let error = run_test_with_overrides(&descriptor, &EnvOverrides::default())
            .expect_err("invalid descriptor must fail");
        let rendered = format!("{error:?}");

        assert!(rendered.contains("test configuration failed"));
        assert!(rendered.contains("invalid registered test name `Invalid name`"));
    }

    #[test]
    fn failure_debug_output_is_human_readable() {
        let failure = TestFailure::configuration("counter-random", "invalid environment");
        let rendered = format!("{failure:?}");

        assert!(rendered.contains("VVM test `counter-random` failed"));
        assert!(rendered.contains("invalid environment"));
    }

    #[test]
    fn trace_root_creation_error_is_reported() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let parent = directory.path().join("blocked");
        fs::create_dir_all(&parent)?;
        let file = parent.join("leaf");
        let _file = File::create(&file)?;
        let impossible_child = file.join("nested");

        let error = ensure_trace_root("counter-random", &impossible_child);

        assert!(
            matches!(error, Err(ref failure) if format!("{failure:?}").contains("failed to create trace directory"))
        );
        Ok(())
    }
}
