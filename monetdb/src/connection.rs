use log::debug;
use std::result;
use url::Url;

use crate::monetizer;
use mapi::errors::MonetDBError;
use mapi::mapi::{MapiConnection, MapiConnectionParams, MapiLanguage};

pub type Result<T> = result::Result<T, MonetDBError>;

/// This implements the connection to a MonetDB database
pub struct Connection {
    _server_url: String,
    connection: MapiConnection,
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
        let mapi_params = MapiConnectionParams::new(
            db,
            parsed.username(),
            parsed.password(),
            Some(MapiLanguage::Sql),
            parsed.host_str(),
            parsed.port(),
        );
        Ok(Connection {
            _server_url: String::from(url),
            connection: MapiConnection::connect(mapi_params)?,
        })
    }

    pub fn get_mapi_connection(&mut self) -> &mut MapiConnection {
        &mut self.connection
    }

    pub fn execute(&mut self, query: &str, params: Vec<monetizer::SQLParameter>) -> Result<u64> {
        let escaped_query = monetizer::apply_parameters(query, params);
        let command = String::from("s") + &escaped_query + "\n;";
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

    // TODO: maybe return a T that represents the type?
    pub fn query(&mut self, query: &str, params: Vec<monetizer::SQLParameter>) -> Result<Vec<String>> {
        let escaped_query = monetizer::apply_parameters(query, params);
        let command = String::from("s") + &escaped_query + "\n;";
        let resp = self.connection.cmd(&command[..])?;

        let response_lines = resp.lines();

        let response_header = response_lines.skip(2);
        let response_body = response_header.skip(3);

        let mut output: Vec<String> = Vec::new();

        for line in response_body {
            let sanitized = line.replace(&['\t', '[', ']', ' '], "");
            let mut splitted = sanitized.split(',').map(|x| x.to_string()).collect::<Vec<String>>();

            output.append(&mut splitted);
        }

        Ok(output)
    }
}
