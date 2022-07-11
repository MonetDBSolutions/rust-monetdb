// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2022 MonetDB B.V.
//

mod integration_tests;
mod monetizer;

use log::debug;
use std::result;
use url::Url;

use mapi::errors::MonetDBError;

pub type Result<T> = result::Result<T, MonetDBError>;

/// This implements the connection to a MonetDB database
pub struct Connection {
    _server_url: String,
    connection: mapi::MapiConnection,
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
        let mapi_params = mapi::MapiConnectionParams::new(
            db,
            parsed.username(),
            parsed.password(),
            Some(mapi::MapiLanguage::Sql),
            parsed.host_str(),
            parsed.port(),
        );
        Ok(Connection {
            _server_url: String::from(url),
            connection: mapi::MapiConnection::connect(mapi_params)?,
        })
    }

    pub fn get_mapi_connection(&mut self) -> &mut mapi::MapiConnection {
        &mut self.connection
    }

    pub fn execute(&mut self, query: &str /*, params: &[&str]*/) -> Result<u64> {
        let command = String::from("s") + query + "\n;";
        let resp = self.connection.cmd(&command[..])?;

        debug!("Query:\n{}\nResponse:\n{}", query, resp);

        let insertions = resp
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse::<u64>()
            .unwrap();
        Ok(insertions)
    }
}
