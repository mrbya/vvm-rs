//! Command-line parsing for `cargo-vvm`.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Removes Cargo's exact injected `vvm` argument while preserving all other arguments.
pub fn normalize_arguments(arguments: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut arguments = arguments.into_iter();
    let mut normalized = Vec::new();

    if let Some(program) = arguments.next() {
        normalized.push(program);
    }

    if let Some(first) = arguments.next() {
        if first != "vvm" {
            normalized.push(first);
        }
    }

    normalized.extend(arguments);
    normalized
}

/// Top-level Cargo subcommand parser.
#[derive(Debug, Parser)]
#[command(
    name = "cargo-vvm",
    bin_name = "cargo vvm",
    about = "VVM functional coverage tools"
)]
pub struct Cli {
    /// Selected VVM operation.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Supported operations.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Runs Cargo tests, merges per-test artifacts, and writes coverage reports.
    Coverage(CoverageCommand),
}

/// Functional-coverage command options.
#[derive(Debug, clap::Args)]
#[command(
    after_help = "The command after -- excludes `cargo`; supported forms are `test ...` and \
                  `nextest run ...`. The default is `test --workspace`. Child status is \
                  preserved, and nextest retries are disabled."
)]
pub struct CoverageCommand {
    /// Exact output root for this isolated run.
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    /// Base filename for merged and rendered outputs.
    #[arg(long, default_value = "coverage")]
    pub(crate) name: String,

    /// Artifact contribution policy.
    #[arg(long, value_enum, default_value_t = CliMergePolicy::PassedOnly)]
    pub(crate) merge_policy: CliMergePolicy,

    /// Amount of rendered bin detail.
    #[arg(long, value_enum, default_value_t = CliBinDetail::Uncovered)]
    pub(crate) bin_detail: CliBinDetail,

    /// Omits per-test provenance from rendered reports.
    #[arg(long)]
    pub(crate) no_inputs: bool,

    /// Displays complete definition fingerprints.
    #[arg(long)]
    pub(crate) fingerprints: bool,

    /// Forwarded Cargo command after `--`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub(crate) child: Vec<OsString>,
}

/// Stable merge-policy spelling.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliMergePolicy {
    /// Include successful tests only.
    PassedOnly,

    /// Include successful and failed tests.
    PassedAndFailed,

    /// Include every test status.
    All,
}

/// Stable report-detail spelling.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliBinDetail {
    /// Omit bin-level details.
    None,

    /// Show uncovered bins.
    Uncovered,

    /// Show all bins.
    All,
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::Parser as _;

    use super::{Cli, Command, normalize_arguments};

    #[test]
    fn normalizes_cargo_injected_vvm_argument() {
        let parsed = normalize_arguments(["cargo-vvm", "vvm", "coverage"].map(OsString::from));

        assert_eq!(parsed, ["cargo-vvm", "coverage"].map(OsString::from));
    }

    #[test]
    fn preserves_direct_invocation_arguments() {
        let parsed = normalize_arguments(["cargo-vvm", "coverage"].map(OsString::from));

        assert_eq!(parsed, ["cargo-vvm", "coverage"].map(OsString::from));
    }

    #[test]
    fn parses_coverage_defaults() {
        let cli = Cli::try_parse_from(["cargo-vvm", "coverage"]);

        assert!(matches!(
            cli,
            Ok(Cli {
                command: Command::Coverage(_)
            })
        ));
    }
}
