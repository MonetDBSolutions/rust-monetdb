pub fn apply_parameters(query: &str, params: Vec<&str>) -> Result<String, String> {
    if params.is_empty() {
        return Ok(query.to_string());
    }

    let split = query.split_inclusive("{}").collect::<Vec<&str>>();

    let mut result: Vec<String> = Vec::new();

    for (i, s) in split.iter().enumerate() {
        let mut temp = String::new();

        if s.contains("{}") {
           let escaped = convert(&params[i]);

           temp = s.to_string().replace("{}", &escaped).to_string();
        }

        result.push(temp);
    }

    let out = result.iter().map(|x| x.to_owned()).collect::<String>();

    Ok(out)
}

fn convert(input: &str) -> String {
    // TODO: we should check which characters we should ban
    format!("'{}'", String::from(input).replace("'", ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stuff_is_escaped_correctly() {
        let input = String::from("foo");
        let target = "'foo'";

        let output = convert(&input);

        assert_eq!(output, target)
    }

    #[test]
    fn stuff_with_ticks_is_escaped_correctly() {
        let input = String::from("'foo'");
        let input2 = String::from("''foo''");
        let input3 = String::from("'foo'''''''");
        let target = "'foo'";

        let output = convert(&input);
        let output1 = convert(&input2);
        let output2 = convert(&input3);

        assert_eq!(output, target);
        assert_eq!(output1, target);
        assert_eq!(output2, target)
    }

    #[test]
    fn query_is_escaped_correctly() {
        let input = "SELECT * FROM foobar WHERE foo = {}";
        let target = "SELECT * FROM foobar WHERE foo = 'bar'";

        let output = apply_parameters(input, vec!["bar"]).unwrap();

        assert_eq!(output, target)
    }
}