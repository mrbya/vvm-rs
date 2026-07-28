use thiserror::Error;

/// Error returned while configuring a clock scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ClockConfigurationError {
    /// A clock name was empty.
    #[error("clock name must not be empty")]
    EmptyName,

    /// Two clocks used the same diagnostic name.
    #[error("clock name `{name}` is already registered")]
    DuplicateName {
        /// Duplicate clock name.
        name: String,
    },
}

impl ClockConfigurationError {
    /// Returns the duplicate name when a name caused this error.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match *self {
            Self::EmptyName => None,
            Self::DuplicateName { ref name } => Some(name),
        }
    }
}
