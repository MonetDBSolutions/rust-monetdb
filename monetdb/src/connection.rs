use log::debug;
use std::result;
use url::Url;

use crate::monetizer;
use crate::row;
use crate::row::MonetType;
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

    #[inline]
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

    pub fn query(&mut self, query: &str, params: Vec<monetizer::SQLParameter>) -> Result<Vec<row::Row>> {
        let escaped_query = monetizer::apply_parameters(query, params);
        let command = String::from("s") + &escaped_query + "\n;";
        let resp = self.connection.cmd(&command[..])?;

        let response_lines = resp.lines();

        let response_header: Vec<String> = response_lines.clone().skip(3).map(|x| self.sanitize(x)).collect();
        let header = self.parse_header(response_header);
        let response_body = response_lines.clone().skip(5);

        let mut output: Vec<row::Row> = Vec::new();

        for line in response_body {
            let sanitized = &self.sanitize(line);
            let splitted: Vec<&str>  = sanitized.split(',').collect();
            let mut out: Vec<MonetType> = Vec::new();

            for (i, v) in splitted.iter().enumerate() {
                let out_type = MonetType::parse(header.get(i).unwrap(), v.trim());

                match out_type {
                    Ok(s) => {
                        out.push(s);
                    }
                    Err(e) => {
                        return Err(e)
                    }
                }
            }

            output.push(row::Row { value: out } );
        }

        Ok(output)
    }

    #[inline]
    fn parse_header(&self, input: Vec<String>) -> Vec<String> {
        let header: Vec<&str> = input[0].split('#').collect();
        header[0].split(',').map(|x| x.trim().to_string()).collect::<Vec<String>>()
    }

    #[inline]
    fn sanitize(&self, line: &str) -> String {
        line.replace(&['\t', '%', '[', ']', ' '], " ")
    }
}
