// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2022 MonetDB B.V.
//
extern crate url;

use std;
use std::fmt;
use std::error::Error;

/// Definition for the low level errors that might occur when talking to a
/// MonetDB server.
#[derive(Debug)]
pub enum MapiError {
    IOError(std::io::Error),
    ConnectionError(String),
    UnimplementedError(String),
    UnknownServerResponse(String),
    OperationError(String),
    ServerError(std::string::FromUtf8Error),
    OtherError(String),
}

impl fmt::Display for MapiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MapiError::*;
        match *self {
            IOError(ref e) => write!(f, "MapiError: {}", e),
            ConnectionError(ref s) => write!(f, "MapiError: Connection error: {}", s),
            UnimplementedError(ref s) => write!(f, "MapiError: Unimplemented MAPI functionality: {}", s),
            UnknownServerResponse(ref s) => {
                write!(f,
                       "MapiError: Server sent something we don't understand: {}",
                       s)
            }
            OperationError(ref s) => write!(f, "MapiError: An error occurred at the server: {}", s),
            ServerError(ref s) => write!(f, "MapiError: Server sent invalid UTF8: {}", s),
            OtherError(ref s) => write!(f, "MapiError: Other error: {}", s),
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

impl From<std::fmt::Error> for MapiError {
    fn from(error: std::fmt::Error) -> Self {
        MapiError::OtherError("Formatting error".to_string())
    }
}

#[derive(Debug)]
pub enum MonetDBError {
    InvalidUrl(url::ParseError),
    UnimplementedError(String),
    ConnectionError(MapiError),
}

impl fmt::Display for MonetDBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MonetDBError::*;
        match *self {
            InvalidUrl(ref e) => write!(f, "MonetDBError: {}", e),
            ConnectionError(ref s) => write!(f, "MonetDBError: ConnectionError: {}", s),
            UnimplementedError(ref s) => write!(f, "MonetDBError: Unimplemented SQL functionality: {}", s),
        }
    }
}

impl Error for MonetDBError {
    fn description(&self) -> &str {
        "MonetDBError"
    }
}

impl From<url::ParseError> for MonetDBError {
    fn from(error: url::ParseError) -> Self {
        MonetDBError::InvalidUrl(error)
    }
}

impl From<MapiError> for MonetDBError {
    fn from(error: MapiError) -> Self {
        MonetDBError::ConnectionError(error)
    }
}
