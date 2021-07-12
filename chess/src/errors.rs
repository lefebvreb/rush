use std::fmt;
use std::num::ParseIntError;

//#################################################################################################
//
//                                  struct ParseFenError
//
//#################################################################################################

/// An error type to handle problems during fen parsing.
#[derive(Debug)]
pub struct ParseFenError {
    msg: &'static str,
}

// ================================ pub(crate) impl

impl ParseFenError {
    // Creates a new parse fen error.
    pub(crate) fn new(msg: &'static str) -> Self {
        ParseFenError {msg}
    }
}

// ================================ traits impl

impl fmt::Display for ParseFenError {
    /// Formats the error message.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while parsing, {}.", self.msg)
    }
}

impl From<ParseIntError> for ParseFenError {
    /// Constructs a parse fen error from an integer parse error.
    fn from(_: ParseIntError) -> Self {
        ParseFenError::new("integer parse error")
    }
}