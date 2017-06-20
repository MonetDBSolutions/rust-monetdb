use std::io;
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::convert::From;

use bytes::Bytes;

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
    pub database:           String,
    pub username:           Option<String>,
    pub password:           Option<String>,
    pub language:           Option<MapiLanguage>,
    pub hostname:           Option<String>,
    pub port:               Option<u32>,
    pub unix_socket:        Option<String>,
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

    pub fn get_bytes(&mut self) -> Result<Bytes, MapiError> {
        let mut buffer = [0; 1024];

        let len = self.socket.read(&mut buffer)?;
        println!("Read {} bytes", len);
        let b = Bytes::from(Vec::from(&buffer[0..len]));
        Ok(b)
    }

    // pub fn shutdown(&self, how: Shutdown) -> Result<(), io::Error> {
    //     self.socket.shutdown(how);
    // }
}

impl From<io::Error> for MapiError {
    fn from(error: io::Error) -> Self {
        MapiError::IOError(error)
    }
}
