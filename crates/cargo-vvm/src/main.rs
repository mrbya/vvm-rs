//! Cargo-facing orchestration for VVM functional coverage.

pub mod cli;
pub mod command;
pub mod error;
pub mod metadata;
pub mod orchestration;
pub mod output;
pub mod process;

use std::process::ExitCode;

use clap::Parser as _;

/// Runs the Cargo subcommand.
fn main() -> ExitCode {
    let arguments = cli::normalize_arguments(std::env::args_os());

    let cli = match cli::Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = u8::try_from(error.exit_code()).unwrap_or(1);

            drop(error.print());
            return ExitCode::from(exit_code);
        }
    };

    orchestration::run(cli)
}
