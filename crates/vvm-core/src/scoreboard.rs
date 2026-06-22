use std::fmt;

/// Compares an expected result against an observed result.
pub trait Scoreboard<E, O> {
    /// Error returned when comparison fails.
    type Error;

    /// Checks one expected and observed result.
    ///
    /// # Errors
    ///
    /// Returns a structured comparison failure.
    fn check(&mut self, expected: E, observed: O) -> Result<(), Self::Error>;
}

/// Exact-equality scoreboard.
///
/// This implementation accepts matching values and retains differring values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExactScoreboard;

/// Exact comparison failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch<E, O> {
    /// Expected value.
    expected: E,

    /// Observed value.
    observed: O,
}

impl<E, O> Mismatch<E, O> {
    /// Return the expected value.
    #[must_use]
    pub const fn expected(&self) -> &E {
        &self.expected
    }

    /// Returns the observed value.
    #[must_use]
    pub const fn observed(&self) -> &O {
        &self.observed
    }

    /// Consumes the mismatch and returns both retained values.
    #[must_use]
    pub fn into_parts(self) -> (E, O) {
        (self.expected, self.observed)
    }
}

impl<E, O> fmt::Display for Mismatch<E, O>
where
    E: fmt::Debug,
    O: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scoreboard mismatch: expected {:?}, observed {:?}",
            self.expected, self.observed
        )
    }
}

impl<E, O> std::error::Error for Mismatch<E, O>
where
    E: fmt::Debug,
    O: fmt::Debug,
{
}

impl<T> Scoreboard<T, T> for ExactScoreboard
where
    T: PartialEq,
{
    type Error = Mismatch<T, T>;

    fn check(&mut self, expected: T, observed: T) -> Result<(), Self::Error> {
        if expected == observed {
            return Ok(());
        }

        Err(Mismatch { expected, observed })
    }
}
