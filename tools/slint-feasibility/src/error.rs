use std::fmt;

/// One actionable reason a feasibility replay check refused its input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckError {
    message: String,
}

impl CheckError {
    /// Construct an actionable fail-closed diagnostic without an input value.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CheckError {}
