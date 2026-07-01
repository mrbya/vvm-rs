use core::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use chrono::{DateTime, Local};
use clap::Parser;
use vvm_core::{
    ReplayToken, Seed, TestDescriptor, TestRegistry, TestRegistryError, TestRun, TestRunConfig,
};

/// VVM Test cli runner.
#[derive(Debug, Parser)]
pub struct TestCli {
    /// Filter registered tests to run.
    #[arg(value_name = "FILTER")]
    filter: Option<String>,

    /// Lists registered tests without executing one.
    #[arg(short, long, conflicts_with_all = ["filter", "seed", "replay", "trace_dir"])]
    list: bool,

    /// Uses the current VVM random algorithm with this seed.
    #[arg(short, long, value_name = "SEED", conflicts_with = "replay")]
    seed: Option<Seed>,

    /// Replays the exact deterministic random stream.
    #[arg(short, long, value_name = "TOKEN", conflicts_with = "seed")]
    replay: Option<ReplayToken>,

    /// Writes waveform output to this path.
    #[arg(short, long, value_name = "PATH")]
    trace_dir: Option<PathBuf>,

    /// Passes in requested number of cycles to test.
    #[arg(short, long, value_name = "CYCLES")]
    cycles: Option<u64>,
}

/// Result of a runner test pass.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RunnerResult {
    /// Total tests registered in the test registry.
    total: usize,

    /// Number of passing tests.
    passed: usize,

    /// Number of failed tests.
    failed: usize,
}

impl RunnerResult {
    /// Constructs new runner result with total number of tests in registry.
    #[must_use]
    pub const fn new(total: usize) -> Self {
        Self {
            total,
            passed: 0,
            failed: 0,
        }
    }

    /// Increments passed tests.
    #[must_use]
    pub const fn bump_passed(mut self) -> Self {
        self.passed = self.passed.saturating_add(1);
        self
    }

    /// Increments failed tests.
    #[must_use]
    pub const fn bump_failed(mut self) -> Self {
        self.failed = self.failed.saturating_add(1);
        self
    }

    /// Returns number of failed tests.
    #[must_use]
    pub const fn failed(&self) -> usize {
        self.failed
    }
}

impl fmt::Display for RunnerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Summary: {} tests run: {} passed, {} failed, {} skipped",
            self.passed.saturating_add(self.failed),
            self.passed,
            self.failed,
            self.total
                .saturating_sub(self.passed.saturating_add(self.failed))
        )
    }
}

impl TestCli {
    /// VVM CLI test runner.
    #[must_use]
    pub fn run(tests: &'static [TestDescriptor]) -> ExitCode {
        let args = Self::parse();
        let time = Local::now();

        if args.list {
            for test in tests {
                println!("{test}");
            }
            return ExitCode::SUCCESS;
        }

        match execute_all(&args, tests, time) {
            Ok(result) => {
                if result.failed() != 0 {
                    eprintln!("{result}");
                    return ExitCode::FAILURE;
                }
                println!("{result}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        }
    }
}

/// Execute all tests.
fn execute_all(
    args: &TestCli,
    tests: &'static [TestDescriptor],
    time: DateTime<Local>,
) -> Result<RunnerResult, TestRegistryError> {
    let registry = TestRegistry::new(tests)?;

    // Filter out test to run if provided using a cli arg
    // This way if no test is provided, all tests are ran.
    let tests_to_run: Vec<&TestDescriptor> = tests
        .iter()
        .filter(|test| {
            args.filter
                .as_deref()
                .is_none_or(|name| test.name().starts_with(name))
        })
        .collect();

    if let Some(ref name) = args.filter
        && tests_to_run.is_empty()
    {
        return Err(TestRegistryError::UnknownTest {
            name: name.to_owned(),
        });
    }

    let mut result = RunnerResult::new(tests.len());

    for test in tests_to_run {
        match execute(test, args, time, &registry) {
            Ok(run) => {
                if run.passed() {
                    println!("{run}");
                    result = result.bump_passed();
                } else {
                    eprintln!("{run}");
                    result = result.bump_failed();
                }
            }

            Err(error) => {
                eprintln!("{error}");
                result = result.bump_failed();
            }
        }
    }

    Ok(result)
}

/// Executes a single test.
fn execute<'a>(
    test: &'a TestDescriptor,
    args: &TestCli,
    time: DateTime<Local>,
    registry: &TestRegistry<'a>,
) -> Result<TestRun<'a>, TestRegistryError> {
    let mut config = TestRunConfig::new();

    if test.capabilities().replay() {
        if let Some(seed) = args.seed {
            config = config.with_replay_token(ReplayToken::new(seed));
        }

        if let Some(replay) = args.replay {
            config = config.with_replay_token(replay);
        }

        if config.replay_token().is_none() {
            if let Some(replay) = test.default_replay_token() {
                config = config.with_replay_token(replay);
            } else {
                config =
                    config.with_replay_token(ReplayToken::new(Seed::from(time.timestamp_millis())));
            }
        }
    }

    if test.capabilities().cycles()
        && let Some(cycles) = args.cycles
    {
        config = config.with_cycles(cycles);
    }

    if test.capabilities().trace() {
        let mut trace_path = PathBuf::from(format!(
            "vvm-trace/{}/{}.vcd",
            time.format("%Y-%m-%d_%H-%M-%S"),
            test.name()
        ));

        if let Some(trace_dir) = args.trace_dir.as_ref() {
            trace_path = trace_dir.join(format!("{}.vcd", test.name()));
            config = config.with_trace_path(&trace_path);
        }

        if config.trace_path().is_none() {
            config = config.with_trace_path(&trace_path);
        }

        let Some(trace_dir) = trace_path.parent() else {
            return Err(TestRegistryError::InvalidTracePath {
                name: test.name(),
                path: trace_path,
            });
        };

        if trace_dir.exists() && !trace_dir.is_dir() {
            return Err(TestRegistryError::TraceDirIsFile {
                name: test.name(),
                path: trace_dir.to_path_buf(),
            });
        }

        if !trace_dir.exists() {
            match fs::create_dir_all(trace_dir) {
                Ok(()) => {}
                Err(error) => {
                    return Err(TestRegistryError::Io {
                        name: test.name(),
                        path: trace_dir.to_path_buf(),
                        source: error.to_string(),
                    });
                }
            }
        }
    }

    let run = registry.run(test.name(), &config)?;

    if run.passed() {
        return Ok(run);
    }

    Err(TestRegistryError::TestFailed {
        name: run.test().name(),
        report: run.outcome().report().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::TestCli;

    #[test]
    fn parses_replay_token() -> Result<(), Box<dyn std::error::Error>> {
        let cli = TestCli::try_parse_from([
            "counter",
            "counter-random",
            "--replay",
            "chacha8-v1:0123456789abcdef",
        ])?;

        assert_eq!(cli.replay, Some("chacha8-v1:0123456789abcdef".parse()?));

        Ok(())
    }
}
