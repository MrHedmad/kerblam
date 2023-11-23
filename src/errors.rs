/// Represents a critical Kerblam! error.
///
/// If this is returned, something has happened so that Kerblam! has to stop,
/// but gracefully.
#[derive(Debug, Clone)]
pub struct StopError {
    /// A message to show the user before stopping.
    pub msg: String,
}

// This should implement std::error::Error and Display. Then you can use this as
// a Result<T, StopError>
