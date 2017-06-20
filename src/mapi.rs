extern crate bytes;

use std::io;
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::convert::From;

pub enum MapiLanguage {
    Sql,
    Mapi,
    Control
}

pub struct MapiConnection {
    language: MapiLanguage,
    socket: Box<Read>
}

#[derive(Debug)]
pub enum MapiError{
    IOError(io::Error),
}

impl fmt::Display for MapiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use mapi::MapiError::IOError;
        match *self {
            IOError(ref e) => write!(f, "MapiError: {}", e)
        }
    }
}

impl Error for MapiError {
    fn description(&self) -> &str {
        "MapiError"
    }
}

pub struct MapiConnectionParams {
    database:           String,
    username:           Option<String>,
    password:           Option<String>,
    language:           Option<MapiLanguage>,
    hostname:           Option<String>,
    port:               Option<u32>,
    unix_socket:        Option<String>,
}

impl MapiConnectionParams {
    pub fn new(database: String) -> MapiConnectionParams {
        MapiConnectionParams {
            database:           database,
            username:           Some(String::from("monetdb")),
            password:           Some(String::from("monetdb")),
            language:           Some(MapiLanguage::Sql),
            hostname:           Some(String::from("localhost")),
            port:               Some(50000),
            unix_socket:        None
        }
    }
}

impl MapiConnection {
    pub fn connect(params: MapiConnectionParams) -> Result<MapiConnection, MapiError> {
        let port = match params.port {
            Some(p) => p,
            None    => 50000
        };

        let mut socket = match params.unix_socket {
            Some(p) => p,
            None    => format!("/tmp/.s.monetdb.{}", port)
        };

        let hostname = match params.hostname {
            Some(h) => {
                if h.starts_with("/") {
                    socket = format!("{}/.s.monetdb.{}", h, port);
                    None
                }
                else {
                    Some(format!("{}:{}", h, port))
                }
            }
            None    => Some(format!("localhost:{}", port))
        };

        let socket = Path::new(&socket);

        let lang = match params.language {
            Some(l) => l,
            None    => MapiLanguage::Sql
        };

        let connection = match hostname {
            Some(h) => {
                println!("connecting to {}", h);
                Box::new(TcpStream::connect(h)?) as Box<Read>
            },
            None    => {
                println!("connecting to socket {}", socket.display());
                Box::new(UnixStream::connect(socket)?) as Box<Read>
            }
        };

        Ok(MapiConnection {
            socket: connection,
            language: lang
        })

    }

    pub fn get_bytes(&mut self) -> Result<String, MapiError> {
        let mut buffer = [0; 1024];

        let len = self.socket.read(&mut buffer)?;
        // TODO: change this
        let s = String::from_utf8(Vec::from(&buffer[0..len])).unwrap();
        Ok(s)
    }
}

impl From<io::Error> for MapiError {
    fn from(error: io::Error) -> Self {
        MapiError::IOError(error)
    }
}
