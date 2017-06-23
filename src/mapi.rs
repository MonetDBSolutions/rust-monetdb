use std::io;
use std::result;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::convert::From;

// use bytes::{Bytes, BytesMut};
use crypto_hash::{Algorithm, hex_digest};

use errors::MapiError;
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
    pub fn new(database: &str) -> MapiConnectionParams {
        MapiConnectionParams {
            database:           database.to_string(),
            username:           Some(String::from("monetdb")),
            password:           Some(String::from("monetdb")),
            language:           Some(MapiLanguage::Sql),
            hostname:           Some(String::from("localhost")),
            port:               Some(50000),
            unix_socket:        None
        }
    }
}

const BLOCK_SIZE: usize = (8*1024 - 2);

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

type Result<T> = result::Result<T, MapiError>;
use bytes::Bytes;
impl MapiConnection {
    pub fn connect(params: MapiConnectionParams) -> Result<MapiConnection> {
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

    fn login(&mut self) -> Result<()> {
        let challenge = self.get_block()?;
        println!("Challenge: {}", String::from_utf8(challenge.to_vec()).unwrap());
        let response = self.challenge_response(&challenge)?;
        self.put_block(response);

        println!("get prompt");
        let prompt = self.get_block()?;
        println!("get prompt");

        println!("{:?}", prompt);
        Ok(())
    }

    fn challenge_response(&mut self, challenge: &Bytes) -> Result<Bytes> {
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

        // if endian == "LIT" {
        //     self.endian = LittleEndian;
        // } else {
        //     self.endian = BiGEndian;
        // }

        let algorithm = self.get_encoding_algorithm(&*algo)?;

        let hash_list: Vec<&str>= hashes.split_terminator(',').collect();
        let hash_algo = self.get_hash_algorithm(hash_list)?;

        let password = hex_digest(algorithm, password.as_bytes());
        let hashed_passwd = hex_digest(hash_algo.1, format!("{}{}", password, salt).as_bytes());

        let ret = format!("BIG:{}:{}{}:{}:{}:", self.username,
                          hash_algo.0, hashed_passwd, self.language,
                          self.database);
        println!("Response = {}", ret);

        Ok(Bytes::from(ret))


    }

    fn get_encoding_algorithm(&self, algo: &str) -> Result<Algorithm> {
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

    fn get_hash_algorithm(&self, algs: Vec<&str>) -> Result<(String, Algorithm)> {
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

    fn get_block(&mut self) -> Result<Bytes> {
        use bytes::{LittleEndian, IntoBuf, Buf};
        let mut buff = vec![];
        if self.language == MapiLanguage::Control /*  && local */{
            // TODO: implement local control
            return Err(MapiError::UnimplementedError("E01".to_string()));
        } else {
            let mut last = false;
            while !last {
                // Header is 2 bytes: The length of the block (maximum value is
                // BLOCK_SIZE) left shifted by 1. If this is the last block of
                // the message then the LSB of the header is set.

                // TODO: Need to control the endianess based on what the server sent
                let header = get_bytes(&mut self.socket, 2)?.into_buf().get_u16::<LittleEndian>();
                let length = header >> 1;
                if header & 1 == 1 {
                    last = true;
                }
                let mut cbuff = get_bytes(&mut self.socket, length as u64)?;
                buff.append(&mut cbuff);
            }
        }
        Ok(Bytes::from(buff))
    }

    fn put_block(&mut self, message: Bytes) -> Result<()> {
        use bytes::LittleEndian;
        if self.language == MapiLanguage::Control /*  && local */ {
            // TODO: implement local control
            return Err(MapiError::UnimplementedError("E02".to_string()));
        } else {
            use bytes::BufMut;
            let mut sl_start = 0;
            let mut sl_end = 0;
            while sl_end + BLOCK_SIZE < message.len() {
                sl_start = sl_end;
                sl_end = sl_end + BLOCK_SIZE;
                let slice = message.slice(sl_start, sl_end);
                let mut header = vec![];
                header.put_u16::<LittleEndian>((slice.len() << 1) as u16);
                self.socket.write_all(header.as_slice())?;
                self.socket.write_all(slice.as_ref())?;
            }

            if message.len() - sl_end > 0 {
                sl_start = sl_end;
                let slice = message.slice_from(sl_start);
                let mut header = vec![];
                header.put_u16::<LittleEndian>(((slice.len() << 1) + 1) as u16);
                self.socket.write_all(header.as_slice())?;
                self.socket.write_all(slice.as_ref())?;
            }
        }
        Ok(())
    }

}

fn get_bytes<R>(stream: R, limit: u64) -> Result<Vec<u8>>
    where R: Read
{
    let mut buff = vec![];
    let mut chunk = stream.take(limit);
    let mut count = limit;
    while count > 0 {
        let mut cbuff = vec![];
        let recv = chunk.read_to_end(&mut cbuff)?;
        if recv == 0 {
            return Err(MapiError::ConnectionError("Server closed the connection".to_string()))
        }
        count -= recv as u64;
        buff.append(&mut cbuff);
    }

    Ok(buff)
}


