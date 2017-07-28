// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2017 MonetDB B.V.
//
extern crate bytes;
extern crate crypto_hash;
extern crate url;
#[macro_use]
extern crate log;

use std::result;
use url::Url;

pub mod mapi;
pub mod errors;

use errors::MonetDBError;

pub type Result<T> = result::Result<T, MonetDBError>;

pub struct Connection {
    server_url: String,
    connection: mapi::MapiConnection
}

impl Connection {
    pub fn connect(url: &str) -> Result<Connection> {
        let parsed = Url::parse(url)?;
        debug!("parsed url {} to", url);
        debug!("  scheme: {}", parsed.scheme());
        debug!("  username: {}", parsed.username());
        debug!("  password: {:?}", parsed.password());
        debug!("  host: {:?}", parsed.host_str());
        debug!("  host: {:?}", parsed.port());
        debug!("  path: {:?}", parsed.path());

        // Remove the initial '/'
        let db = parsed.path().get(1..).unwrap();
        let mapi_params = mapi::MapiConnectionParams::new(db,
                                                          parsed.username(),
                                                          parsed.password(),
                                                          Some( mapi::MapiLanguage::Sql ),
                                                          parsed.host_str(),
                                                          parsed.port());
        Ok(Connection {
            server_url: String::from(url),
            connection: mapi::MapiConnection::connect(mapi_params)?
        })
    }
}
