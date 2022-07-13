use std::fmt;

pub struct SQLParameters {
    value: String
}

impl From<&str> for SQLParameters {
    fn from(input: &str) -> Self {
        SQLParameters { value: format!("'{}'", String::from(input).replace('\'', "")) }
    }
}

impl From<i8> for SQLParameters {
    fn from(input: i8) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<u8> for SQLParameters {
    fn from(input: u8) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<i16> for SQLParameters {
    fn from(input: i16) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<u16> for SQLParameters {
    fn from(input: u16) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<i32> for SQLParameters {
    fn from(input: i32) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<u32> for SQLParameters {
    fn from(input: u32) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<i64> for SQLParameters {
    fn from(input: i64) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}

impl From<u64> for SQLParameters {
    fn from(input: u64) -> Self {
        SQLParameters { value: int_to_string(input) }
    }
}


impl fmt::Display for SQLParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
} 

pub fn apply_parameters(query: &str, parameters: &[SQLParameters]) -> Option<String> {
    if parameters.is_empty() {
        return Some(query.to_string());
    }

    let split = query.split_inclusive("{}").collect::<Vec<&str>>();

    let mut result: Vec<String> = Vec::new();

    for (i, s) in split.iter().enumerate() {
        let mut temp = String::new();

        if s.contains("{}") {
           temp = s.replace("{}", &format!("{}", parameters[i]));
        }

        result.push(temp);
    }

    let out = result.iter().map(|x| x.to_owned()).collect::<String>();

    Some(out)
}

fn int_to_string<T: fmt::Display>(arg: T) -> String {
    arg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

   #[test]
    fn ints_are_escaped_correctly() {
        let input = SQLParameters::from(10);
        let input1 = SQLParameters::from(999);

        assert_eq!(format!("{input}"), "(10)");
        assert_eq!(format!("{input1}"), "(999)");
    }

    #[test]
    fn strings_are_escaped_correctly() {
        let input = SQLParameters::from("foo");
        let input_with_ticks = SQLParameters::from("'foo'");
        let input_with_loads_of_ticks = SQLParameters::from("'''foo'''''");

        assert_eq!(format!("{input}"), "'foo'");
        assert_eq!(format!("{input_with_ticks}"), "'foo'");
        assert_eq!(format!("{input_with_loads_of_ticks}"), "'foo'");
    }

    #[test]
    fn queries_are_escaped_correctly() {
        let q1 = apply_parameters("SELECT * FROM foo WHERE bar = {}", &[SQLParameters::from("foobar")]).unwrap();
        let q2 = apply_parameters("SELECT * FROM foo WHERE bar = {} AND baz = {}", &[SQLParameters::from("foobar"), 
        SQLParameters::from("something cool")]).unwrap();

        assert_eq!(q1, String::from("SELECT * FROM foo WHERE bar = 'foobar'"));
        assert_eq!(q2, String::from("SELECT * FROM foo WHERE bar = 'foobar' AND baz = 'something cool'"));
    }

    #[test]
    fn queries_with_ints_are_escaped_correctly() {
        let q1 = apply_parameters("SELECT * FROM foo WHERE bar = {}", &[SQLParameters::from(10)]).unwrap();
        let q2 = apply_parameters("SELECT * FROM foo WHERE bar = {} AND baz = {}", &[SQLParameters::from(1), SQLParameters::from(2)]).unwrap();
        let q3 = apply_parameters("SELECT * FROM foo WHERE bar = {} AND baz = {}", &[SQLParameters::from(1), SQLParameters::from("foo")]).unwrap();

        assert_eq!(q1, String::from("SELECT * FROM foo WHERE bar = (10)"));
        assert_eq!(q2, String::from("SELECT * FROM foo WHERE bar = (1) AND baz = (2)"));
        assert_eq!(q3, String::from("SELECT * FROM foo WHERE bar = (1) AND baz = 'foo'"));
    }
}
