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
    pub metadata: QueryMetadata,
    pub result: Vec<Row>
}

#[derive(PartialEq, Debug)]
pub struct QueryMetadata {
    pub response_type: ResponseType,
    pub result_id: i32,
    pub number_of_rows: i32,
    pub column_count: i32,
    pub number_of_rows_in_message: i32,
    pub query_id: i32,
    pub query_time: i32,
    pub mal_optimizer_time: i32,
    pub sql_optimizer_time: i32,
}

impl QueryResponse {
    pub fn new(resp: String) -> Result<QueryResponse> {
        let mut response_lines = resp.lines();

        let metadata_header = match response_lines.next() {
            Some(s) => s,
            None => return Err(MonetDBError::UnimplementedError(String::from("Received no metadata")))
        };

        let metadata = QueryResponse::parse_metadata_header(&metadata_header);

        let result = match QueryResponse::parse_response_output(response_lines) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        Ok(QueryResponse {
            metadata,
            result
        })
    }
    
    fn parse_response_output(response_lines: std::str::Lines) -> Result<Vec<Row>> {
            let response_header: Vec<String> = response_lines.clone().skip(2).map(String::from).collect();
            let header = QueryResponse::parse_header(response_header);
            let response_body = response_lines.clone().skip(4);

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
        fn parse_metadata_header(input: &str) -> QueryMetadata {
            let header = input.to_string(); 
            let splitted: Vec<&str> = header.split(' ').collect();

            let response_type: ResponseType = match splitted[0] {
                "&1" => ResponseType::Data,
                _ => ResponseType::Data
            };

            QueryMetadata {
                response_type,
                result_id: splitted[1].parse().unwrap(),
                number_of_rows: splitted[2].parse().unwrap(),
                column_count: splitted[3].parse().unwrap(),
                number_of_rows_in_message: splitted[4].parse().unwrap(),
                query_id: splitted[5].parse().unwrap(),
                query_time: splitted[6].parse().unwrap(),
                mal_optimizer_time: splitted[7].parse().unwrap(),
                sql_optimizer_time: splitted[8].parse().unwrap(),
            }
        }

        #[inline]
        fn parse_header(input: Vec<String>) -> Vec<String> {
            let header: Vec<&str> = input[0].split('#').collect();
            header[0].split(',').map(|x| x.replace('%', " ").trim().to_string()).collect::<Vec<String>>()
        }

        #[inline]
        fn sanitize(line: &str) -> String {
            let mut temp = String::from(line);
            temp.pop();
            temp.remove(0);

            temp
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

            let metadata = QueryMetadata {
                 response_type: ResponseType::Data,
                 result_id: 0,
                 number_of_rows: 2,
                 column_count: 2,
                 number_of_rows_in_message: 2,
                 query_id: 1443,
                 query_time: 1918,
                 mal_optimizer_time: 479,
                 sql_optimizer_time: 178,
            };

            let actual = QueryResponse::new(response).unwrap();

            let desired = QueryResponse {
                 metadata,
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

            let metadata = QueryMetadata {
                 response_type: ResponseType::Data,
                 result_id: 0,
                 number_of_rows: 2,
                 column_count: 1,
                 number_of_rows_in_message: 2,
                 query_id: 1496,
                 query_time: 896,
                 mal_optimizer_time: 1242,
                 sql_optimizer_time: 55,
           };

            let actual = QueryResponse::new(response).unwrap();

            let desired = QueryResponse {
                 metadata,
                 result: vec![
                    Row { value: vec![MapiString("And He said: \\\"Let there be Light!\\\"".to_string())] },
                    Row { value: vec![MapiString("Very hard string: [%]".to_string())] },
                 ]
            };

            assert_eq!(actual, desired);
        }

}
