use std::fmt;

pub struct SQLParameters {
    value: String
}

impl From<&str> for SQLParameters {
    fn from(input: &str) -> Self {
        SQLParameters { value: String::from(input).replace('\'', "") }
    }
}

impl fmt::Display for SQLParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.value)
    }
} 

pub fn apply_parameters(query: &str, parameters: &[&str]) -> Option<String> {
    if parameters.is_empty() {
        return Some(query.to_string());
    }

    let split = query.split_inclusive("{}").collect::<Vec<&str>>();

    let mut result: Vec<String> = Vec::new();

    for (i, s) in split.iter().enumerate() {
        let mut temp = String::new();

        if s.contains("{}") {
           let escaped = SQLParameters::from(parameters[i]);

           temp = s.replace("{}", &format!("{escaped}"));
        }

        result.push(temp);
    }

    let out = result.iter().map(|x| x.to_owned()).collect::<String>();

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let q1 = apply_parameters("SELECT * FROM foo WHERE bar = {}", &["foobar"]).unwrap();
        let q2 = apply_parameters("SELECT * FROM foo WHERE bar = {} AND baz = {}", &["foobar", "something cool"]).unwrap();

        assert_eq!(q1, String::from("SELECT * FROM foo WHERE bar = 'foobar'"));
        assert_eq!(q2, String::from("SELECT * FROM foo WHERE bar = 'foobar' AND baz = 'something cool'"));
    }

}