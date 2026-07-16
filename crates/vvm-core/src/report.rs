use std::fmt;

use crate::TestResult;

/// Compact one-line view of a testbench result.
#[derive(Debug, Clone, Copy)]
pub struct TestSummary<'a, S, F, E> {
    /// Structured result being formatted.
    result: &'a TestResult<S, F, E>,
}

impl<'a, S, F, E> TestSummary<'a, S, F, E> {
    /// Creates a compact result view.
    pub(crate) const fn new(result: &'a TestResult<S, F, E>) -> Self {
        Self { result }
    }
}

impl<S, F, E> fmt::Display for TestSummary<'_, S, F, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.result;

        let status = if result.passed() { "PASS" } else { "FAIL" };

        let cycles = result.cycles();
        let checks = result.checks();
        let failures = result.failure_count();

        write!(
            formatter,
            "{status}: {cycles} cycle{}, {checks} check{}, {failures} check failure{}, time \
             {}..{} ticks",
            plural_suffix(cycles == 1),
            plural_suffix(checks == 1),
            plural_suffix(failures == 1),
            result.start_time().ticks(),
            result.final_time().ticks(),
        )?;

        if result.stopped_by_failure_policy() {
            formatter.write_str(", stopped by failure policy")?;
        }

        if let Some(error) = result.simulation_error() {
            if let Some(clock_name) = error.clock_name() {
                write!(formatter, ", simulation error on clock `{clock_name}`")?;
            } else {
                formatter.write_str(", simulation error")?;
            }
        }

        if result.finalization_error().is_some() {
            formatter.write_str(", finalization error")?;
        }

        if let Some(replay) = result.replay_token() {
            write!(formatter, ", replay token {replay}")?;
        }

        Ok(())
    }
}

/// Detailed, multiline view of a testbench result.
#[derive(Debug, Clone, Copy)]
pub struct DetailedTestReport<'a, S, F, E> {
    /// Structured result being formatted.
    result: &'a TestResult<S, F, E>,
}

impl<'a, S, F, E> DetailedTestReport<'a, S, F, E> {
    /// Creates a detailed result view.
    pub(crate) const fn new(result: &'a TestResult<S, F, E>) -> Self {
        Self { result }
    }
}

impl<S, F, E> fmt::Display for DetailedTestReport<'_, S, F, E>
where
    S: fmt::Debug,
    F: fmt::Display,
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.result;

        write!(formatter, "{}", result.summary())?;

        if !result.failures().is_empty() {
            write!(
                formatter,
                "\n\nCheck failures ({}):",
                result.failure_count(),
            )?;

            for (number, failure) in (1_u64..).zip(result.failures()) {
                write!(
                    formatter,
                    "\n\n  [{number}] primary cycle {} at {}",
                    failure.cycle(),
                    failure.time(),
                )?;

                write!(formatter, "\n      stimulus: {:?}", failure.stimulus())?;

                write!(formatter, "\n      error: {}", failure.error())?;
            }
        }

        if let Some(error) = result.simulation_error() {
            write!(
                formatter,
                "\n\nSimulation error:\n\n  primary cycle: {}\n  time: {}\n  stage: {}",
                error.cycle(),
                error.time(),
                error.stage(),
            )?;

            if let Some(clock_name) = error.clock_name() {
                write!(formatter, "\n  clock: {clock_name}")?;
            }

            write!(formatter, "\n  error: {}", error.source_error())?;
        }

        if let Some(error) = result.finalization_error() {
            write!(
                formatter,
                "\n\nFinalization error:\n\n  time: {}\n  error: {}",
                result.final_time(),
                error,
            )?;
        }

        Ok(())
    }
}

/// Returns an empty suffix for one item and `"s"` otherwise.
const fn plural_suffix(is_one: bool) -> &'static str {
    if is_one { "" } else { "s" }
}
