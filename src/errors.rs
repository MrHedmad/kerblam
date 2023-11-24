use std::{fmt::Display, error::Error};

#[derive(Debug, Clone)]
/// Represents a critical Kerblam! error.
///
/// If this is returned, something has happened so that Kerblam! has to stop,
/// but gracefully.
pub struct StopError {
    /// A message to show the user before stopping.
    pub msg: String,
}

impl<T: Into<String>> From<T> for StopError {
    fn from(value: T) -> Self {
        StopError {msg : value.into()}
    }
}

impl Error for StopError {
   // It's blanket implemented already! 
}

impl Display for StopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

