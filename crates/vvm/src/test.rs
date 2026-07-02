use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};
use std::{env, fmt, fs};

use vvm_core::{ReplayToken, Seed, TestDescriptor, TestRun, TestRunConfig};

/// Environment variable selecting a seed-derived replay token.
const ENV_SEED: &str = "VVM_SEED";
/// Environment variable selecting an explicit replay token.
const ENV_REPLAY: &str = "VVM_REPLAY";
/// Environment variable overriding replayable test cycle counts.
const ENV_CYCLES: &str = "VVM_CYCLES";
/// Environment variable selecting the root directory for trace files.
const ENV_TRACE_DIR: &str = "VVM_TRACE_DIR";

/// Runs one VVM descriptor through the standard Rust test harness bridge.
///
/// # Errors
///
/// Returns a formatted test failure when configuration or execution fails.
pub fn run_test(descriptor: &TestDescriptor) -> Result<(), TestFailure> {
    let overrides = EnvOverrides::parse(descriptor.name())?;
    let config = overrides.config_for(descriptor)?;
    let run = descriptor
        .run(&config)
        .map_err(|error| TestFailure::configuration(descriptor.name(), error.to_string()))?;

    if run.passed() {
        return Ok(());
    }

    Err(TestFailure::from_run(&run))
}

/// Human-readable failure returned from generated standard Rust tests.
pub struct TestFailure {
    /// Fully formatted failure report.
    message: String,
}

impl TestFailure {
    /// Creates a configuration or setup failure.
    fn configuration(name: &str, details: impl Into<String>) -> Self {
        Self {
            message: format!("VVM test `{name}` failed\n\n{}", details.into()),
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
}

impl fmt::Debug for TestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl fmt::Display for TestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TestFailure {}

/// Parsed environment overrides shared across all standard test wrappers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EnvOverrides {
    /// Global replay override, if any.
    replay_token: Option<ReplayToken>,

    /// Global cycle-count override, if any.
    cycles: Option<u64>,

    /// Root directory for generated waveform files, if explicitly configured.
    trace_root: Option<PathBuf>,
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
                format!(
                    "environment variables `{ENV_SEED}` and `{ENV_REPLAY}` are mutually exclusive"
                ),
            ));
        }

        let replay_token = if let Some(value) = replay {
            Some(value.parse::<ReplayToken>().map_err(|error| {
                TestFailure::configuration(
                    test_name,
                    format!("invalid `{ENV_REPLAY}` value `{value}`: {error}"),
                )
            })?)
        } else if let Some(value) = seed {
            let parsed_seed = value.parse::<Seed>().map_err(|error| {
                TestFailure::configuration(
                    test_name,
                    format!("invalid `{ENV_SEED}` value `{value}`: {error}"),
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

        Ok(Self {
            replay_token,
            cycles,
            trace_root,
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
}

/// Returns the default trace root for this process.
fn default_trace_root() -> PathBuf {
    static RUN_ID: OnceLock<String> = OnceLock::new();

    let run_id = RUN_ID.get_or_init(|| {
        env::var("NEXTEST_RUN_ID").unwrap_or_else(|_| {
            let elapsed = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos();

            format!("pid-{}-{elapsed:032x}", std::process::id())
        })
    });

    PathBuf::from("target").join("vvm-trace").join(run_id)
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

    use vvm_core::{TestCapabilities, TestOutcome};

    use super::{
        ENV_CYCLES, ENV_REPLAY, ENV_SEED, ENV_TRACE_DIR, EnvOverrides, TestFailure,
        default_trace_root, ensure_trace_root,
    };
    use crate::{ReplayToken, Seed, TestDescriptor};

    fn descriptor(capabilities: TestCapabilities) -> TestDescriptor {
        TestDescriptor::new(
            "counter-random",
            "Counter random",
            |_config| TestOutcome::error("not used"),
            capabilities,
        )
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
