// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2018 MonetDB B.V.
//


//! The implementation of the low level connection to MonetDB.
use std::io;
use std::result;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;

use crypto_hash::{Algorithm, hex_digest};
use crate::errors::MapiError;

/// This enum specifies the different languages that the protocol can handle.
#[derive(PartialEq)]
pub enum MapiLanguage {
    Sql,
    Mapi,
    Control,
}

impl fmt::Display for MapiLanguage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MapiLanguage::Sql => write!(f, "sql"),
            MapiLanguage::Mapi => write!(f, "mapi"),
            MapiLanguage::Control => write!(f, "control"),
        }
    }
}

/// The connection parameters for a mapi connection.
pub struct MapiConnectionParams {
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub language: Option<MapiLanguage>,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub unix_socket: Option<String>,
}

impl MapiConnectionParams {
    /// Create a new set of connection parameters.
    pub fn new(database: &str, username: &str,
               password: Option<&str>, language: Option<MapiLanguage>,
               hostname: Option<&str>, port: Option<u16>) -> MapiConnectionParams {
        MapiConnectionParams {
            database: database.to_string(),
            username: Some(
                if username == ""
                {
                    String::from("monetdb")
                } else {
                    username.to_string()
                }),
            password: Some(password.unwrap_or("monetdb").to_string()),
            language: Some(language.unwrap_or(MapiLanguage::Sql)),
            hostname: Some(hostname.unwrap_or("localhost").to_string()),
            port: Some(port.unwrap_or(50000)),
            unix_socket: None,
        }
    }
}


// MAPI Protocol version 9: Server and client exchange information in blocks of
// 8094 bytes.
const BLOCK_SIZE: usize = (8 * 1024 - 2);

/// Low level connection to MonetDB. This struct implements the mapi protocol version 9.
#[allow(dead_code)]
pub struct MapiConnection {
    hostname: String,
    username: String,
    password: String,
    database: String,
    port: u16,
    language: MapiLanguage,
    socket: MapiSocket,
    state: MapiConnectionState,
}


type Result<T> = result::Result<T, MapiError>;
impl MapiConnection {
    /// Establish a mapi connection given a set of connection params.
    pub fn connect(params: MapiConnectionParams) -> Result<MapiConnection> {
        let port = params.port.unwrap_or(50000);

        let mut socket = match params.unix_socket {
            Some(p) => p,
            None => format!("/tmp/.s.monetdb.{}", port),
        };

        let hostname = match params.hostname {
            Some(h) => {
                if h.starts_with("/") {
                    socket = format!("{}/.s.monetdb.{}", h, port);
                    None
                } else {
                    Some(format!("{}:{}", h, port))
                }
            }
            None => Some(format!("localhost:{}", port)),
        };

        let socket = Path::new(&socket);

        let lang = params.language.unwrap_or(MapiLanguage::Sql);

        let socket = match hostname.clone() {
            Some(h) => MapiSocket::TCP(TcpStream::connect(h)?),
            None => {
                let sbuf = [b'0'; 1];
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
            hostname: hostname.unwrap_or(String::from("localhost")),
            username: params.username.unwrap_or(String::from("monetdb")),
            password: params.password.unwrap_or(String::from("monetdb")),
            database: params.database,
            port: params.port.unwrap_or(50000),
            state: MapiConnectionState::StateInit,
        };

        connection.login(0)?;
        connection.state = MapiConnectionState::StateReady;

        Ok(connection)
    }

    /// Send a command to the server
    pub fn cmd(&mut self, operation: &str) -> Result<String> {
        use self::ServerResponsePrompt::*;
        match self.state {
            MapiConnectionState::StateInit => {
                return Err(MapiError::ConnectionError("Not connected".to_string()))
            }
            MapiConnectionState::StateReady => {
                self.put_block(operation.as_bytes().to_vec())?;
                let response = self.get_block()?;
                let (prompt, prompt_length) = MapiConnection::parse_prompt(&response)?;

                match prompt {
                    MsgPrompt => return Ok("".to_string()),
                    MsgOk => {
                        let resp = response.split_at(prompt_length).1;
                        return Ok(String::from_utf8(resp.to_vec())?);
                    }
                    // Tell the server it's not getting anything more from us
                    MsgMore => return self.cmd(""),
                    MsgQ(p) => {
                        match p {
                            QResponse::QUpdate => {
                                // TODO: find a way to remove this clone
                                for line in String::from_utf8(response.clone())?.lines() {
                                    if line.starts_with("!") {
                                        return Err(MapiError::OperationError(line.to_string()));
                                    }
                                }
                                return Ok(String::from_utf8(response)?);
                            }
                            _ => {
                                return Ok(String::from_utf8(response)?);
                            }

                        }
                    }
                    MsgHeader => {
                        return Ok(String::from_utf8(response)?);
                    }
                    MsgTuple => {
                        return Ok(String::from_utf8(response)?);
                    }
                    MsgError => {
                        let er = String::from_utf8(response)?;
                        return Err(MapiError::OperationError(er));
                    }

                    _ => {
                        return Err(MapiError::ConnectionError(format!("E05 (cmd unimplemented handling of: {:?})",
                                                                      prompt)))
                    }
                }

            }
        }
    }

    fn login(&mut self, iteration: u8) -> Result<()> {
        debug!("Starting login dance");
        use self::ServerResponsePrompt::*;

        let challenge = self.get_block()?;
        debug!("Server sent: {}", String::from_utf8(challenge.clone())?);
        let response = self.challenge_response(&challenge)?;
        self.put_block(response)?;

        let mut response = self.get_block()?;
        let (prompt, prompt_length) = MapiConnection::parse_prompt(&response)?;

        match prompt {
            MsgPrompt => return Ok(()), // Server is happy
            MsgOk => return Ok(()), // Server is happy
            MsgError => {
                return Err(MapiError::ConnectionError(format!("login: Server error: {}", String::from_utf8(response)?)))
            }
            MsgRedirect => {
                let redirect = response.split_off(prompt_length);
                let mut iter = redirect.split(|x| *x == b':');
                let prot = String::from_utf8_lossy(iter.nth(1).unwrap());
                debug!("prot = {}", prot);
                if prot == "merovingian" {
                    debug!("Restarting authentication");
                    return self.login(iteration + 1);
                } else if prot == "monetdb" {
                    return Err(MapiError::UnimplementedError("E03 (unimplemented redirect)".to_string()));
                } else {
                    return Err(MapiError::ConnectionError(format!( "Unknown redirect: {}", String::from_utf8_lossy(redirect.as_ref() ))));
                }
            }
            _ => {
                return Err(MapiError::UnknownServerResponse(format!("login: server responded with {:?} during login", prompt)))
            }
        }

    }

    fn parse_prompt(bytes: &Vec<u8>) -> Result<(ServerResponsePrompt, usize)> {
        use self::ServerResponsePrompt::*;
        use bytes::{Buf, IntoBuf};
        use self::QResponse::*;

        let mut buf = bytes.into_buf();

        if buf.remaining() == 0 {
            Ok((MsgPrompt, 0))
        } else {
            let initial_byte = buf.get_u8();
            if initial_byte == b'#' {
                Ok((MsgInfo, 1))
            } else if initial_byte == b'!' {
                Ok((MsgError, 1))
            } else if initial_byte == b'%' {
                Ok((MsgHeader, 1))
            } else if initial_byte == b'[' {
                Ok((MsgTuple, 1))
            } else if initial_byte == b'^' {
                Ok((MsgRedirect, 1))
            } else if initial_byte == 1 {
                let byte = buf.get_u8();
                if byte == 2 {
                    let byte = buf.get_u8();
                    if byte == b'\n' {
                        Ok((MsgMore, 3))
                    } else {
                        Err(MapiError::UnknownServerResponse(format!("parse_prompt: Invalid More prompt: \\1\\2{}", byte)))
                    }
                } else {
                    Err(MapiError::UnknownServerResponse(format!("parse_prompt: Invalid More prompt: \\1{}", byte)))
                }
            } else if initial_byte == b'&' {
                let byte = buf.get_u8();
                if byte == b'1' {
                    Ok((MsgQ(QTable), 2))
                } else if byte == b'2' {
                    Ok((MsgQ(QUpdate), 2))
                } else if byte == b'3' {
                    Ok((MsgQ(QSchema), 2))
                } else if byte == b'4' {
                    Ok((MsgQ(QTrans), 2))
                } else if byte == b'5' {
                    Ok((MsgQ(QPrepare), 2))
                } else if byte == b'6' {
                    Ok((MsgQ(QBlock), 2))
                } else {
                    Err(MapiError::UnknownServerResponse(format!("parse_prompt: Invalid Q: &{}", byte)))
                }
            } else if initial_byte == b'=' {
                if buf.get_u8() == b'O' && buf.get_u8() == b'K' {
                    Ok((MsgOk, 3))
                } else {
                    Ok((MsgTupleNoSclice, 1))
                }
            } else {
                Err(MapiError::UnknownServerResponse(format!("parse_prompt: Invalid prompt: Byte[0] = {}", initial_byte)))
            }
        }
    }

    fn challenge_response(&mut self, challenge: &Vec<u8>) -> Result<Vec<u8>> {
        let mut iter = challenge.split(|x| *x == b':');

        let salt = String::from_utf8_lossy(iter.next().unwrap());
        let identity = String::from_utf8_lossy(iter.next().unwrap());
        let protocol = String::from_utf8_lossy(iter.next().unwrap());
        let hashes = String::from_utf8_lossy(iter.next().unwrap());
        let _endianess = String::from_utf8_lossy(iter.next().unwrap()); // Unused for now
        let algo = String::from_utf8_lossy(iter.next().unwrap());
        let password = self.password.clone();

        if protocol != "9" {
            return Err(MapiError::ConnectionError(format!("Unsupported protocol version: {}", protocol)));
        }

        if identity != "mserver" && identity != "merovingian" {
            return Err(MapiError::ConnectionError(format!("Unknown server type: {}", identity)));
        }

        let algorithm = self.get_encoding_algorithm(&algo[..])?;

        let hash_list: Vec<&str> = hashes.split_terminator(',').collect();
        let hash_algo = self.get_hash_algorithm(hash_list)?;

        let password = hex_digest(algorithm, password.as_bytes());
        let hashed_passwd = hex_digest(hash_algo.1, format!("{}{}", password, salt).as_bytes());

        let ret = format!("BIG:{}:{}{}:{}:{}:",
                          self.username,
                          hash_algo.0,
                          hashed_passwd,
                          self.language,
                          self.database)
            .as_bytes()
            .to_vec();

        Ok(ret)
    }

    fn get_encoding_algorithm(&self, algo: &str) -> Result<Algorithm> {
        if algo == "MD5" {
            Ok(Algorithm::MD5)
        } else if algo == "SHA1" {
            Ok(Algorithm::SHA1)
        } else if algo == "SHA256" {
            Ok(Algorithm::SHA256)
        } else if algo == "SHA512" {
            Ok(Algorithm::SHA512)
        } else {
            Err(MapiError::ConnectionError(format!("Server requested unsupported cryptographic algorithm {}", algo)))
        }
    }

    fn get_hash_algorithm(&self, algs: Vec<&str>) -> Result<(String, Algorithm)> {
        let ret = if algs.contains(&"SHA512") {
            Some(("{SHA512}".to_string(), Algorithm::SHA512))
        } else if algs.contains(&"SHA256") {
            Some(("{SHA256}".to_string(), Algorithm::SHA256))
        } else if algs.contains(&"SHA1") {
            Some(("{SHA1}".to_string(), Algorithm::SHA1))
        } else if algs.contains(&"MD5") {
            Some(("{MD5}".to_string(), Algorithm::MD5))
        } else {
            None
        };

        match ret {
            Some(algo) => Ok(algo),
            None => {
                Err(MapiError::ConnectionError("No supported hash algorithm found".to_string()))
            }
        }
    }

    fn get_block(&mut self) -> Result<Vec<u8>> {
        use bytes::{IntoBuf, Buf};
        let mut buff = vec![];
        if self.language == MapiLanguage::Control
        // && local
        {
            // TODO: implement local control
            return Err(MapiError::UnimplementedError("E01".to_string()));
        } else {
            let mut last = false;
            while !last {
                // Header is 2 bytes: The length of the block (maximum value is
                // BLOCK_SIZE) left shifted by 1. If this is the last block of
                // the message then the LSB of the header is set.

                // TODO: Need to control the endianess based on what the server sent
                let header = get_bytes(&mut self.socket, 2)?.into_buf().get_u16_le();
                let length = header >> 1;
                if header & 1 == 1 {
                    last = true;
                }
                let mut cbuff = get_bytes(&mut self.socket, length as u64)?;
                buff.append(&mut cbuff);
            }
        }
        Ok(buff)
    }

    fn put_block(&mut self, message: Vec<u8>) -> Result<()> {
        if self.language == MapiLanguage::Control
        // && local
        {
            // TODO: implement local control
            return Err(MapiError::UnimplementedError("E02 (put_block local control language)".to_string()));
        } else {
            use bytes::BufMut;
            let mut sl_start;
            let mut sl_end = 0;
            while sl_end + BLOCK_SIZE < message.len() {
                sl_start = sl_end;
                sl_end = sl_end + BLOCK_SIZE;
                let slice = &message[sl_start..sl_end];
                let mut header = vec![];
                header.put_u16_le((slice.len() << 1) as u16);
                self.socket.write_all(header.as_slice())?;
                self.socket.write_all(slice.as_ref())?;
            }

            if message.len() - sl_end > 0 {
                sl_start = sl_end;
                let slice = &message[sl_start..];
                let mut header = vec![];
                header.put_u16_le(((slice.len() << 1) + 1) as u16);
                self.socket.write_all(header.as_slice())?;
                self.socket.write_all(slice.as_ref())?;
            }
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        match self.socket.shutdown(Shutdown::Both) {
            Ok(()) => return Ok(()),
            Err(e) => Err(MapiError::IOError(e))
        }
    }
}

pub fn get_bytes<R>(stream: R, limit: u64) -> Result<Vec<u8>>
    where R: Read
{
    let mut buff = vec![];
    let mut chunk = stream.take(limit);
    let mut count = limit;
    while count > 0 {
        let mut cbuff = vec![];
        let recv = chunk.read_to_end(&mut cbuff)?;
        if recv == 0 {
            return Err(MapiError::ConnectionError("Server closed the connection".to_string()));
        }
        count -= recv as u64;
        buff.append(&mut cbuff);
    }

    Ok(buff)
}

enum MapiSocket {
    TCP(TcpStream),
    UNIX(UnixStream),
}

impl Read for MapiSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            MapiSocket::TCP(ref mut s) => s.read(buf),
            MapiSocket::UNIX(ref mut s) => s.read(buf),
        }
    }
}

impl Write for MapiSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            MapiSocket::TCP(ref mut s) => s.write(buf),
            MapiSocket::UNIX(ref mut s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            MapiSocket::TCP(ref mut s) => s.flush(),
            MapiSocket::UNIX(ref mut s) => s.flush(),
        }
    }
}

impl MapiSocket {
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match *self {
            MapiSocket::TCP(ref s) => s.shutdown(how),
            MapiSocket::UNIX(ref s) => s.shutdown(how),
        }
    }
}

#[derive(Debug)]
enum ServerResponsePrompt {
    MsgPrompt,
    MsgMore,
    MsgInfo,
    MsgError,
    MsgQ(QResponse),
    MsgHeader,
    MsgTuple,
    MsgTupleNoSclice,
    MsgRedirect,
    MsgOk,
}

#[derive(Debug)]
enum QResponse {
    QTable,
    QUpdate,
    QSchema,
    QTrans,
    QPrepare,
    QBlock,
}

enum MapiConnectionState {
    StateReady,
    StateInit,
}