use core::fmt;

use mapi::errors::MonetDBError;

#[derive(Debug, PartialEq)]
pub enum MonetType {
    Double(f32),
    Int(i32),
    MapiString(String),
}

impl MonetType {
    pub fn parse(_type: &String, s: &str) -> Result<Self, MonetDBError> {
        match _type.as_str() {
            "double" => Ok(MonetType::Double(s.parse::<f32>().unwrap())),
            "int" => Ok(MonetType::Int(s.parse::<i32>().unwrap())),
            "string" => Ok(MonetType::MapiString(String::from(s))),
            "clob" => Ok(MonetType::MapiString(String::from(s))),
            _ => Err(MonetDBError::UnimplementedError(String::from("Unimplemented type")))
        }
    }
}

impl fmt::Display for MonetType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            MonetType::Double(s) => write!(formatter, "{}", s.to_string()),
            MonetType::Int(s) => write!(formatter, "{}", s.to_string()),
            MonetType::MapiString(s) => write!(formatter, "{}", s.to_string()),
        }        
    }
}

#[derive(Debug, PartialEq)]
pub struct Row {
    pub value: Vec<MonetType>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ints_are_parsed_correctly() {
        let input = MonetType::parse(&String::from("int"), "1").unwrap();
        let input1 = MonetType::parse(&String::from("int"), "100").unwrap();
        let input2 = MonetType::parse(&String::from("int"), "999").unwrap();

        assert_eq!(input, MonetType::Int(1));
        assert_eq!(input1, MonetType::Int(100));
        assert_eq!(input2, MonetType::Int(999));
    }

    #[test]
    fn floats_are_parsed_correctly() {
        let input = MonetType::parse(&String::from("double"), "1.0").unwrap();
        let input1 = MonetType::parse(&String::from("double"), "100.9").unwrap();
        let input2 = MonetType::parse(&String::from("double"), "999.9").unwrap();

        assert_eq!(input, MonetType::Double(1.0));
        assert_eq!(input1, MonetType::Double(100.9));
        assert_eq!(input2, MonetType::Double(999.9));
    }

    #[test]
    fn clobs_are_parsed_correctly() {
        let input = MonetType::parse(&String::from("clob"), "foo").unwrap();
        let input1 = MonetType::parse(&String::from("clob"), "bar").unwrap();
        let input2 = MonetType::parse(&String::from("clob"), "999.9").unwrap();
        let input3 = MonetType::parse(&String::from("clob"), "foo bar with a lot of spaces").unwrap();
        let input4 = MonetType::parse(&String::from("clob"), "'''foo bar with a lot of backticks'''").unwrap();
        let input5 = MonetType::parse(&String::from("clob"), "\"Do you like quotes?\"").unwrap();

        assert_eq!(input, MonetType::MapiString(String::from("foo")));
        assert_eq!(input1, MonetType::MapiString(String::from("bar")));
        assert_eq!(input2, MonetType::MapiString(String::from("999.9")));
        assert_eq!(input3, MonetType::MapiString(String::from("foo bar with a lot of spaces")));
        assert_eq!(input4, MonetType::MapiString(String::from("'''foo bar with a lot of backticks'''")));
        assert_eq!(input5, MonetType::MapiString(String::from("\"Do you like quotes?\"")));
    }

    #[test]
    fn strings_are_parsed_correctly() {
        let input = MonetType::parse(&String::from("string"), "foo").unwrap();
        let input1 = MonetType::parse(&String::from("string"), "bar").unwrap();
        let input2 = MonetType::parse(&String::from("string"), "999.9").unwrap();
        let input3 = MonetType::parse(&String::from("string"), "foo bar with a lot of spaces").unwrap();
        let input4 = MonetType::parse(&String::from("string"), "'''foo bar with a lot of backticks'''").unwrap();
        let input5 = MonetType::parse(&String::from("string"), "\"Do you like quotes?\"").unwrap();
        let input6 = MonetType::parse(&String::from("string"), "'And he said: \"Let there be light!\"'").unwrap();
        let input7 = MonetType::parse(&String::from("string"), "Very tricky string: [%]").unwrap();

        assert_eq!(input, MonetType::MapiString(String::from("foo")));
        assert_eq!(input1, MonetType::MapiString(String::from("bar")));
        assert_eq!(input2, MonetType::MapiString(String::from("999.9")));
        assert_eq!(input3, MonetType::MapiString(String::from("foo bar with a lot of spaces")));
        assert_eq!(input4, MonetType::MapiString(String::from("'''foo bar with a lot of backticks'''")));
        assert_eq!(input5, MonetType::MapiString(String::from("\"Do you like quotes?\"")));
        assert_eq!(input6, MonetType::MapiString(String::from("'And he said: \"Let there be light!\"'")));
        assert_eq!(input7, MonetType::MapiString(String::from("Very tricky string: [%]")));
    }

}
