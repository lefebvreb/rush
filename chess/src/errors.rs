use std::fmt;
use std::num::ParseIntError;

// An error type to handle problems during fen parsing
#[derive(Debug)]
pub struct ParseFenError {
    msg: String,
}

impl ParseFenError {
    // Create a new parse fen error 
    pub fn new<S: ToString>(msg: S) -> ParseFenError {
        ParseFenError {
            msg: msg.to_string()
        }
    }
}

impl fmt::Display for ParseFenError {
    // Display the error
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while parsing fen, {}.", self.msg)
    }
}

impl From<ParseIntError> for ParseFenError {
    // Construct a parse fen error from an integer parse error
    fn from(e: ParseIntError) -> ParseFenError {
        ParseFenError::new(format!("integer parse error: {}", e))
    }
}