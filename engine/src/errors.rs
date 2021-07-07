use std::fmt;

//#################################################################################################
//
//                                  struct EngineError
//
//#################################################################################################

/// An error type to handle problems during fen parsing.
#[derive(Debug)]
pub struct EngineError {
    msg: &'static str,
}

// ================================ pub(crate) impl

impl EngineError {
    // Creates a new parse fen error.
    pub(crate) fn new(msg: &'static str) -> Self {
        EngineError {msg}
    }
}

// ================================ traits impl

impl fmt::Display for EngineError {
    /// Formats the error message.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Engine error, {}.", self.msg)
    }
}