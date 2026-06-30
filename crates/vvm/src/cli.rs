use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use vvm_core::{ReplayToken, Seed, TestDescriptor, TestRegistry, TestRegistryError, TestRunConfig};

/// Akafuka
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

impl TestCli {
    /// Runs VVM test cli.
    #[must_use]
    pub fn run(tests: &'static [TestDescriptor]) -> ExitCode {
        let args = Self::parse();

        if args.list {
            for test in tests {
                println!("{test}");
            }
            return ExitCode::SUCCESS;
        }

        if let Some(trace_dir) = args.trace_dir.as_ref() {
            if !trace_dir.is_dir() {
                eprintln!(
                    "Provided trace output dir `{}` is a file.",
                    trace_dir.display()
                );
                return ExitCode::FAILURE;
            }
            if !trace_dir.exists() {
                match fs::create_dir_all(trace_dir) {
                    Ok(()) => {}
                    Err(error) => {
                        eprintln!(
                            "I/O error when trying to create trace output dir `{}`:\n{error}",
                            trace_dir.display()
                        );
                        return ExitCode::FAILURE;
                    }
                }
            }
        }

        match Self::execute(args, tests) {
            Ok(()) => {}
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }

        ExitCode::SUCCESS
    }

    /// Execute helper placeholder.
    fn execute(args: Self, tests: &'static [TestDescriptor]) -> Result<(), TestRegistryError> {
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

        if let Some(name) = args.filter
            && tests_to_run.is_empty()
        {
            return Err(TestRegistryError::UnknownTest { name });
        }

        tests_to_run.iter().try_for_each(|test| {
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
                        config = config.with_replay_token(ReplayToken::new(Seed::from(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis(),
                        )));
                    }
                }
            }

            if test.capabilities().cycles()
                && let Some(cycles) = args.cycles
            {
                config = config.with_cycles(cycles);
            }

            if test.capabilities().trace()
                && let Some(trace_dir) = args.trace_dir.as_ref()
            {
                let trace_path = trace_dir.join(format!("{}.vcd", test.name()));
                config = config.with_trace_path(trace_path);
            }

            let run = registry.run(test.name(), &config)?;

            if run.passed() {
                println!("{run}");
                println!("test completed successfully");

                return Ok(());
            }

            Err(TestRegistryError::TestFailed {
                name: run.test().name(),
                report: run.outcome().report().to_owned(),
            })
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
