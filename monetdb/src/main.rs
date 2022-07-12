// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2022 MonetDB B.V.
//
extern crate monetdb;
#[macro_use]
extern crate log;
extern crate env_logger;

//use log::LogLevel;

// use monetdb_rust::mapi::*;
use monetdb::*;
use monetizer;

fn main() {
    env_logger::init().unwrap();
    let mut c = Connection::connect("mapi://localhost:50000/demo").unwrap();
    let res = c.execute("CREATE TABLE IF NOT EXISTS foo (i int)", &[]).unwrap();
    info!("Result = {}", res);
    let res = c.execute("INSERT INTO foo VALUES (1), (2)", &[]).unwrap();
    info!("Result = {}", res);
    let res = c.execute("INSERT INTO foo VALUES {}, {}", &[monetizer::SQLParameters::from(3), monetizer::SQLParameters::from(4)]).unwrap();
    info!("Result = {}", res);
    let res = c.execute("SELECT * from foo", &[]).unwrap();
    info!("Result = {}", res);
}
