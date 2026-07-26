//! Small synchronous process boundary for production and unit tests.

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

/// Metadata process request.
#[derive(Debug, Clone)]
pub struct MetadataRequest {
    /// Cargo program path.
    pub(crate) program: OsString,

    /// Optional Cargo toolchain selector.
    pub(crate) toolchain: Option<OsString>,

    /// Optional forwarded manifest path.
    pub(crate) manifest_path: Option<PathBuf>,

    /// Working directory.
    pub(crate) current_dir: PathBuf,
}

/// Captured output from the quiet metadata process.
#[derive(Debug)]
pub struct ProcessOutput {
    /// Process exit status.
    pub(crate) status: ExitStatus,

    /// Captured standard output.
    pub(crate) stdout: Vec<u8>,

    /// Captured standard error.
    pub(crate) stderr: Vec<u8>,
}

/// Live test child request.
#[derive(Debug, Clone)]
pub struct TestProcessRequest {
    /// Cargo program path.
    pub(crate) program: OsString,

    /// Optional Cargo toolchain selector.
    pub(crate) toolchain: Option<OsString>,

    /// Preserved forwarded arguments.
    pub(crate) arguments: Vec<OsString>,

    /// Working directory.
    pub(crate) current_dir: PathBuf,

    /// Absolute per-test artifact directory.
    pub(crate) coverage_dir: PathBuf,

    /// Optional nextest retry override.
    pub(crate) nextest_retries: bool,
}

/// Synchronous process operations needed by orchestration.
pub trait ProcessRunner {
    /// Runs quiet Cargo metadata.
    ///
    /// # Errors
    ///
    /// Returns an operating-system process error.
    fn cargo_metadata(&mut self, request: &MetadataRequest) -> Result<ProcessOutput, io::Error>;

    /// Runs Cargo tests with live standard streams.
    ///
    /// # Errors
    ///
    /// Returns an operating-system process error.
    fn run_tests(&mut self, request: &TestProcessRequest) -> Result<ExitStatus, io::Error>;
}

/// Standard process implementation.
pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    fn cargo_metadata(&mut self, request: &MetadataRequest) -> Result<ProcessOutput, io::Error> {
        let mut command = Command::new(&request.program);

        if let Some(toolchain) = request.toolchain.as_ref() {
            command.arg(toolchain);
        }

        command
            .arg("metadata")
            .args(["--format-version", "1", "--no-deps"]);

        if let Some(path) = request.manifest_path.as_ref() {
            command.arg("--manifest-path").arg(path);
        }

        let output = command.current_dir(&request.current_dir).output()?;

        Ok(ProcessOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    fn run_tests(&mut self, request: &TestProcessRequest) -> Result<ExitStatus, io::Error> {
        let mut command = Command::new(&request.program);

        if let Some(toolchain) = request.toolchain.as_ref() {
            command.arg(toolchain);
        }

        command
            .args(&request.arguments)
            .current_dir(&request.current_dir)
            .env("VVM_COVERAGE_DIR", &request.coverage_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if request.nextest_retries {
            command.env("NEXTEST_RETRIES", "0");
        }

        command.status()
    }
}
