use std;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum MapiError{
    IOError(std::io::Error),
    ConnectionError(String),
    UnimplementedError(String),
    UnknownServerResponse(String)
}

impl fmt::Display for MapiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MapiError::*;
        match *self {
            IOError(ref e) => write!(f, "MapiError: {}", e),
            ConnectionError(ref s) => write!(f, "MapiError: Connection error: {}", s),
            UnimplementedError(ref s) => write!(f, "MapiError: Unimplemented functionality: {}", s),
            UnknownServerResponse(ref s) => write!(f, "MapiError: Server sent something we don't understand: {}", s),
        }
    }
}

impl Error for MapiError {
    fn description(&self) -> &str {
        "MapiError"
    }
}

impl From<std::io::Error> for MapiError {
    fn from(error: std::io::Error) -> Self {
        MapiError::IOError(error)
    }
}
