use std::result;
use crate::row::{ Row, MonetType };
use mapi::errors::MonetDBError;

// TODO: add more response types:
// Redirect, Modification, etc...
#[derive(PartialEq, Debug)]
pub enum ResponseType {
    Data
}


pub type Result<T> = result::Result<T, MonetDBError>;

#[derive(PartialEq, Debug)]
pub struct QueryResponse {
    pub response_type: ResponseType,
    pub result_id: i32,
    pub number_of_rows: i32,
    pub column_count: i32,
    pub number_of_rows_in_message: i32,
    pub query_id: i32,
    pub query_time: i32,
    pub mal_optimizer_time: i32,
    pub sql_optimizer_time: i32,
    pub result: Vec<Row>
}

impl QueryResponse {
    pub fn new(resp: String) -> Result<QueryResponse> {
        let response_lines: Vec<&str> = resp.lines().collect();
        let metadata_header = match response_lines.get(0) {
            Some(s) => s,
            None => return Err(MonetDBError::UnimplementedError(String::from("Received no metadata")))
        };

        let metadata_header = QueryResponse::parse_metadata_header(&metadata_header);

        let result_rows = match QueryResponse::parse_response_output(resp) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        Ok(QueryResponse {
            response_type: metadata_header.0,
            result_id: metadata_header.1,
            number_of_rows: metadata_header.2,
            column_count: metadata_header.3,
            number_of_rows_in_message: metadata_header.4,
            query_id: metadata_header.5,
            query_time: metadata_header.6,
            mal_optimizer_time: metadata_header.7,
            sql_optimizer_time: metadata_header.8,
            result: result_rows
        })
    }
    
    fn parse_response_output(resp: String) -> Result<Vec<Row>> {
            let response_lines = resp.lines();

            let response_header: Vec<String> = response_lines.clone().skip(3).map(|x| QueryResponse::sanitize(x)).collect();
            let header = QueryResponse::parse_header(response_header);
            let response_body = response_lines.clone().skip(5);

            let mut output: Vec<Row> = Vec::new();

            for line in response_body {
                let sanitized = QueryResponse::sanitize(line);
                let splitted: Vec<&str>  = sanitized.split(',').collect();
                let mut out: Vec<MonetType> = Vec::new();

                for (i, v) in splitted.iter().enumerate() {
                    let value = QueryResponse::remove_first_and_last_quote(v.trim());
                    let out_type = MonetType::parse(header.get(i).unwrap(), value.trim());

                    match out_type {
                        Ok(s) => {
                            out.push(s);
                        }
                        Err(e) => {
                            return Err(e)
                        }
                    }
                }

                output.push(Row { value: out } );
            }

            Ok(output)
        }

        #[inline]
        fn parse_metadata_header(input: &str) -> (ResponseType, i32, i32, i32, i32, i32, i32, i32, i32) {
            let header = input.to_string(); 
            let splitted: Vec<&str> = header.split(' ').collect();

            let response_type: ResponseType = match splitted[0] {
                "&1" => ResponseType::Data,
                _ => ResponseType::Data
            };

            (
                response_type,
                splitted[1].parse().unwrap(),
                splitted[2].parse().unwrap(),
                splitted[3].parse().unwrap(),
                splitted[4].parse().unwrap(),
                splitted[5].parse().unwrap(),
                splitted[6].parse().unwrap(),
                splitted[7].parse().unwrap(),
                splitted[8].parse().unwrap(),
            )
        }

        #[inline]
        fn parse_header(input: Vec<String>) -> Vec<String> {
            let header: Vec<&str> = input[0].split('#').collect();
            header[0].split(',').map(|x| x.trim().to_string()).collect::<Vec<String>>()
        }

        #[inline]
        fn sanitize(line: &str) -> String {
            line.replace(&['\t', '%', '[', ']'], " ")
        }
        
        #[inline]
        fn remove_first_and_last_quote( line: &str) -> String {
            let first_char = line.chars().take(1).last().unwrap();

            if first_char == '\"' {
                let mut x = line.to_string();
                x.pop();
                x.remove(0);
                return x;
            }

            line.to_string()
        }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::MonetType::{MapiString, Int };

    #[test]
    fn test_valid_query_response_1() {
            let response = r###"&1 0 2 2 2 1443 1918 479 178
% sys.foo4,	sys.foo4 # table_name
% i,	x # name
% int,	clob # type
% 1,	3 # length
[ 1,	"foo"	]
[ 2,	"bar"	]"###.to_string();

            let actual = QueryResponse::new(response).unwrap();

            let desired = QueryResponse {
                 response_type: ResponseType::Data,
                 result_id: 0,
                 number_of_rows: 2,
                 column_count: 2,
                 number_of_rows_in_message: 2,
                 query_id: 1443,
                 query_time: 1918,
                 mal_optimizer_time: 479,
                 sql_optimizer_time: 178,
                 result: vec![
                    Row { value: vec![Int(1), MapiString("foo".to_string())] },
                    Row { value: vec![Int(2), MapiString("bar".to_string())] },
                 ]
            };

            assert_eq!(actual, desired);
        }

    #[test]
    fn test_valid_query_response_with_weird_chars() {
            let response = r###"&1 0 2 1 2 1496 896 1242 55
% sys.quotes # table_name
% x # name
% clob # type
% 34 # length
[ "And He said: \"Let there be Light!\""	]
[ "Very hard string: [%]"	]"###.to_string();


            let actual = QueryResponse::new(response).unwrap();

            let desired = QueryResponse {
                 response_type: ResponseType::Data,
                 result_id: 0,
                 number_of_rows: 2,
                 column_count: 1,
                 number_of_rows_in_message: 2,
                 query_id: 1496,
                 query_time: 896,
                 mal_optimizer_time: 1242,
                 sql_optimizer_time: 55,
                 result: vec![
                    Row { value: vec![MapiString("And He said: \\\"Let there be Light!\\\"".to_string())] },
                    Row { value: vec![MapiString("Very hard string: [%]".to_string())] },
                 ]
            };

            assert_eq!(actual, desired);
        }

}
