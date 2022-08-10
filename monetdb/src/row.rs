#[derive(Debug, PartialEq)]
pub enum MonetType {
    Double(f32),
    Int(i32),
    MapiString(String),
}

impl MonetType {
    pub fn parse(_type: &String, s: &str) -> Option<Self> {
        match _type.as_str() {
            "double" => Some(MonetType::Double(s.parse::<f32>().unwrap())),
            "int" => Some(MonetType::Int(s.parse::<i32>().unwrap())),
            "string" => Some(MonetType::MapiString(String::from(s).replace('"', ""))),
            "clob" => Some(MonetType::MapiString(String::from(s).replace('"', ""))),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Row {
    pub value: Vec<Option<MonetType>>
}

