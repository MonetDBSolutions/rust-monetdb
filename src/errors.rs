/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0.  If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Copyright 1997 - July 2008 CWI, August 2008 - 2017 MonetDB B.V.
 */
use std;
use std::fmt;
use std::error::Error;

/// Definition for the low level errors that might occur when talking to a
/// MonetDB server.
#[derive(Debug)]
pub enum MapiError{
    IOError(std::io::Error),
    ConnectionError(String),
    UnimplementedError(String),
    UnknownServerResponse(String),
    OperationError(String),
    ServerError(std::string::FromUtf8Error)
}

impl fmt::Display for MapiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MapiError::*;
        match *self {
            IOError(ref e) => write!(f, "MapiError: {}", e),
            ConnectionError(ref s) => write!(f, "MapiError: Connection error: {}", s),
            UnimplementedError(ref s) => write!(f, "MapiError: Unimplemented functionality: {}", s),
            UnknownServerResponse(ref s) => write!(f, "MapiError: Server sent something we don't understand: {}", s),
            OperationError(ref s) => write!(f, "MapiError: An error occurred at the server: {}", s),
            ServerError(ref s) => write!(f, "MapiError: Server sent invalid UTF8: {}", s),
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

impl From<std::string::FromUtf8Error> for MapiError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        MapiError::ServerError(error)
    }
}
