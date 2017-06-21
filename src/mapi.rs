use std::io;
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::convert::From;

// use bytes::{Bytes, BytesMut};
use bytes::Bytes;
use crypto_hash::{Algorithm, hex_digest};

#[derive(PartialEq)]
pub enum MapiLanguage {
    Sql,
    Mapi,
    Control
}

impl fmt::Display for MapiLanguage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MapiLanguage::Sql     => write!(f, "sql"),
            MapiLanguage::Mapi    => write!(f, "mapi"),
            MapiLanguage::Control => write!(f, "control")
        }
    }
}

pub enum MapiSocket {
    TCP(TcpStream),
    UNIX(UnixStream)
}

impl Read for MapiSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            MapiSocket::TCP(ref mut s) => s.read(buf),
            MapiSocket::UNIX(ref mut s) => s.read(buf)
        }
    }
}

impl Write for MapiSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            MapiSocket::TCP(ref mut s) => s.write(buf),
            MapiSocket::UNIX(ref mut s) => s.write(buf)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            MapiSocket::TCP(ref mut s) => s.flush(),
            MapiSocket::UNIX(ref mut s) => s.flush()
        }
    }
}

#[allow(dead_code)]
pub struct MapiConnection {
    hostname: String,
    username: String,
    password: String,
    database: String,
    port: u32,
    language: MapiLanguage,
    socket: MapiSocket
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

const BLOCK: usize = (8*1024 - 2);

impl MapiConnection {
    pub fn connect(params: MapiConnectionParams) -> Result<MapiConnection, MapiError> {
        let port = match params.port {
            Some(p) => p,
            None    => 50000
        };

        let mut socket = match params.unix_socket {
            Some(p) => format!("{}", p),
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

        let socket = match hostname.clone() {
            Some(h) => {
                MapiSocket::TCP(TcpStream::connect(h)?)
            },
            None    => {
                let sbuf  = [b'0'; 1];
                let mut c = UnixStream::connect(socket)?;
                // We need to send b'0' to initialize the connection
                if lang != MapiLanguage::Control {
                    c.write(&sbuf).unwrap();
                }
                MapiSocket::UNIX(c)
            }
        };
        let mut connection = MapiConnection {
            socket: socket,
            language: lang,
            hostname: match hostname {
                Some(val) => val,
                None      => String::from("localhost")
            },
            username: match params.username {
                Some(val) => val,
                None      => String::from("monetdb")
            },
            password: match params.password {
                Some(val) => val,
                None      => String::from("monetdb")
            },
            database: params.database,
            port: match params.port {
                Some(val) => val,
                None      => 50000
            }
        };

        connection.login();

        Ok(connection)
    }

    pub fn get_bytes(&mut self) -> Result<Bytes, MapiError> {
        let mut buffer = [0; 1024];

        let len = self.socket.read(&mut buffer)?;
        println!("Read {} bytes", len);
        let b = Bytes::from(Vec::from(&buffer[0..len]));
        Ok(b)
    }

    fn login(&mut self) -> Result<(), MapiError> {
        let challenge = self.get_block()?;
        let response = self.challenge_response(&challenge)?;
        self.put_block(response);

        println!("get prompt");
        let prompt = self.get_block()?;
        println!("get prompt");

        println!("{:?}", prompt);
        Ok(())
    }

    fn put_block(&mut self, message: Bytes) -> Result<(), MapiError> {
        self.socket.write(message.as_ref());
        Ok(())
    }
    fn challenge_response(&mut self, challenge: &Bytes) -> Result<Bytes, MapiError> {
        let mut iter = challenge.split(|x| *x == b':');

        let salt     = String::from_utf8_lossy(iter.next().unwrap());
        let identity = String::from_utf8_lossy(iter.next().unwrap());
        let protocol = String::from_utf8_lossy(iter.next().unwrap());
        let hashes   = String::from_utf8_lossy(iter.next().unwrap());
        let endian   = String::from_utf8_lossy(iter.next().unwrap());
        let algo     = String::from_utf8_lossy(iter.next().unwrap());
        let password = self.password.clone();

        if protocol != "9" {
            return Err(MapiError::ConnectionError(format!("Unsupported protocol version: {}", protocol)))
        }

        if identity != "mserver" && identity != "merovingian" {
            return Err(MapiError::ConnectionError(format!("Unknown server type: {}", identity)))
        }

        let algorithm = self.get_encoding_algorithm(&*algo)?;

        let hash_list: Vec<&str>= hashes.split_terminator(',').collect();
        let hash_algo = self.get_hash_algorithm(hash_list)?;

        let password = hex_digest(algorithm, self.password.as_bytes());
        let hashed_passwd = hex_digest(hash_algo.1, format!("{}{}", password, salt).as_bytes());

        let ret = format!("BIG:{}:{}{}:{}:{}", self.username, hash_algo.0, hashed_passwd, self.language, self.database);
        println!("Response = {}", ret);

        Ok(Bytes::from(ret))


    }

    fn get_encoding_algorithm(&self, algo: &str) -> Result<Algorithm, MapiError> {
        if algo == "MD5" {
            Ok(Algorithm::MD5)
        } else if algo == "SHA1" {
            Ok( Algorithm::SHA1 )
        } else if algo == "SHA256" {
            Ok( Algorithm::SHA256 )
        } else if algo == "SHA512" {
            Ok( Algorithm::SHA512 )
        } else {
            Err(MapiError::ConnectionError(format!("Server requested unsupported cryptographic algorithm {}", algo)))
        }
    }

    fn get_hash_algorithm(&self, algs: Vec<&str>) -> Result<(String, Algorithm), MapiError> {
        for hash in algs {
            if hash == "SHA1" {
                return Ok(("{SHA1}".to_string(), Algorithm::SHA1));
            } else if hash == "MD5" {
                return Ok(("{MD5}".to_string(), Algorithm::MD5));
            } else {
                ;
            }
        }
        return Err(MapiError::ConnectionError("No supported hash algorithm found".to_string()));

    }

    fn get_block(&mut self) -> Result<Bytes, MapiError> {
        let mut buffer = [0; BLOCK];
        let len: usize = self.socket.read(&mut buffer)?;
        Ok(Bytes::from(Vec::from(&buffer[0..len])))
    }
}

#[derive(Debug)]
pub enum MapiError{
    IOError(io::Error),
    ConnectionError(String)
}

impl fmt::Display for MapiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use mapi::MapiError::*;
        match *self {
            IOError(ref e) => write!(f, "MapiError: {}", e),
            ConnectionError(ref s) => write!(f, "MapiError: Connection error: {}", s)
        }
    }
}

impl Error for MapiError {
    fn description(&self) -> &str {
        "MapiError"
    }
}

impl From<io::Error> for MapiError {
    fn from(error: io::Error) -> Self {
        MapiError::IOError(error)
    }
}
