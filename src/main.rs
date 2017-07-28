// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0.  If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright 1997 - July 2008 CWI, August 2008 - 2017 MonetDB B.V.
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
    // let c = Connection::connect("mapi://monetdb:monetdb@localhost:50000/marvin");
    let mut c = Connection::connect("mapi://localhost:50000/marvin").unwrap();
    let res = c.execute("INSERT INTO foo VALUES (1), (a);").unwrap();
    info!("Inserted {} values", res);
    // let mut mc = c.get_mapi_connection();

    // let res = mc.cmd("sINSERT INTO foo VALUES (1), (2);").unwrap();
    // debug!("Response =\n{}", res);
    // debug!("Connecting to local merovignian using MAPI");
    // let dparams = mapi::MapiConnectionParams::new("marvin");

    // let mut c1 = mapi::MapiConnection::connect(dparams).unwrap();
    // let res1 = c1.cmd("sINSERT INTO foo VALUES (1),('a');");
    // match res1 {
    //     Ok(p) => debug!("Response = {}", p),
    //     Err(e) => debug!("Error = {}", e),
    // };
    // let res1 = c1.cmd("sSELECT * from foo;").unwrap();
    // debug!("Response =\n{}", res1);
    // let ds = c1.get_bytes().unwrap();

    // println!("Daemon sent: {:?}", ds);

    // println!();
    // println!("Connecting to local mserver...");
    // let mut sparams = MapiConnectionParams::new("foo");
    // sparams.port = Some(30000);

    // let mut c2 = MapiConnection::connect(sparams).unwrap();
    // println!("connected");
    // let res2 = c2.cmd("sSELECT * from _tables limit 3;").unwrap();
    // println!("Response = {}", res2);
    // let ss = c2.get_bytes().unwrap();

    // println!("Server sent: {:?}", ss);

    // println!();
    // println!("Connecting to merovingian through unix socket");
    // let mut uparams = MapiConnectionParams::new("marvin");
    // uparams.hostname = Some(String::from("/tmp"));

    // let mut c3 = MapiConnection::connect(uparams).unwrap();
    // let us = c3.get_bytes().unwrap();

    // println!("Socket sent: {:?}", us);

    // println!();
    // println!("Connecting to remote merovingian");
    // let mut rparams = MapiConnectionParams::new("SF-10");
    // rparams.hostname = Some(String::from("marvinaws"));

    // let mut c3 = MapiConnection::connect(rparams).unwrap();
}
