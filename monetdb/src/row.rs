#[derive(Debug)]
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
            "string" => Some(MonetType::MapiString(String::from(s))),
            "clob" => Some(MonetType::MapiString(String::from(s))),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Row {
    pub value: Vec<Option<MonetType>>
}

