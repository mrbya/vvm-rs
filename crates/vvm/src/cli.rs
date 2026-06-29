use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use vvm_core::{ReplayToken, Seed, TestDescriptor, TestRegistry, TestRegistryError, TestRunConfig};

/// Akafuka
#[derive(Debug, Parser)]
pub struct TestCli {
    /// Registered test to run.
    #[arg(value_name = "TEST")]
    test: Option<String>,

    /// Lists registered tests without executing one.
    #[arg(short, long, conflicts_with_all = ["test", "seed", "replay", "trace"])]
    list: bool,

    /// Uses the current VVM random algorithm with this seed.
    #[arg(long, value_name = "SEED", conflicts_with = "replay")]
    seed: Option<Seed>,

    /// Replays the exact deterministic random stream.
    #[arg(long, value_name = "TOKEN", conflicts_with = "seed")]
    replay: Option<ReplayToken>,

    /// Writes waveform output to this path.
    #[arg(long, value_name = "PATH")]
    trace: Option<PathBuf>,
}

impl TestCli {
    /// Runs VVM test cli.
    #[must_use]
    pub fn run(tests: &'static [TestDescriptor]) -> ExitCode {
        let args = Self::parse();

        if args.list {
            println!("list placeholder");
            return ExitCode::SUCCESS;
        }

        // Filter out test to run if provided using a cli arg
        // This way if no test is provided, all tests are ran.
        let tests_to_run = tests
            .iter()
            .filter(|test| args.test.clone().is_none_or(|name| name == test.name()));

        let mut config = TestRunConfig::new();

        if let Some(seed) = args.seed {
            config = config.with_replay_token(ReplayToken::new(seed));
        }

        if let Some(replay) = args.replay {
            config = config.with_replay_token(replay);
        }

        if let Some(trace) = args.trace {
            config = config.with_trace_path(trace);
        }

        let mut result = ExitCode::SUCCESS;

        for test in tests_to_run {
            match Self::execute(test.name(), &config, tests) {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("{error}");
                    result = ExitCode::FAILURE;
                }
            }
        }

        result
    }

    /// Execute helper placeholder.
    fn execute(
        test: &'static str,
        config: &TestRunConfig,
        tests: &'static [TestDescriptor],
    ) -> Result<(), TestRegistryError> {
        let run = TestRegistry::new(tests)?.run(test, config)?;

        if run.passed() {
            println!("{run}");
            println!("test completed successfully");

            return Ok(());
        }

        Err(TestRegistryError::TestFailed {
            name: run.test().name(),
            report: run.outcome().report().to_owned(),
        })
    }
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

        assert_eq!(cli.replay, Some("chacha8-v1:0123456789abcdef".parse()?,),);

        Ok(())
    }
}
