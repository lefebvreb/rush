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
    msg: String,
}

// ================================ pub(crate) impl

impl ParseFenError {
    // Creates a new parse fen error.
    pub(crate) fn new<S: ToString>(msg: S) -> ParseFenError {
        ParseFenError {
            msg: msg.to_string()
        }
    }
}

// ================================ traits impl

impl fmt::Display for ParseFenError {
    /// Formats the error message.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while parsing fen, {}.", self.msg)
    }
}

impl From<ParseIntError> for ParseFenError {
    /// Constructs a parse fen error from an integer parse error.
    fn from(e: ParseIntError) -> ParseFenError {
        ParseFenError::new(format!("integer parse error: {}", e))
    }
}