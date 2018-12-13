// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2018 MonetDB B.V.
//
extern crate monetdb_rust;
#[macro_use]
extern crate log;
extern crate env_logger;

use log::LogLevel;

// use monetdb_rust::mapi::*;
use monetdb_rust::*;

fn main() {
    env_logger::init().unwrap();
    let mut c = Connection::connect("mapi://localhost:50000/rustdb").unwrap();
    // let res = c.execute("CREATE TABLE foo (i int)").unwrap();
    let res = c.execute("INSERT INTO foo VALUES (1), (2)").unwrap();
    let res = c.execute("SELECT * from foo").unwrap();
    info!("Inserted {} values", res);
}
