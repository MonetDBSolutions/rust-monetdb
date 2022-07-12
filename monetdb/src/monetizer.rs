pub enum MapiValue {
    MapiString(&'static str),
    MapiInt(i64),
    MapiBool(bool)
}

pub fn apply_parameters(query: &str, params: Vec<MapiValue>) -> Result<String, String> {
    if params.is_empty() {
        return Ok(query.to_string());
    }

    let split = query.split_inclusive("{}").collect::<Vec<&str>>();

    let mut result: Vec<String> = Vec::new();

    for (i, s) in split.iter().enumerate() {
        let mut temp = String::new();
        temp = s.to_string();

        if s.contains("{}") {
           let escaped = convert(&params[i]);

           temp = s.to_string().replace("{}", &escaped).to_string();
        }

        result.push(temp);
    }

    let out = result.iter().map(|x| x.to_owned()).collect::<String>();

    Ok(out)
}

fn convert(input: &MapiValue) -> String {
    match input {
        MapiValue::MapiString(val) => monet_escape(String::from(val.to_owned())),
        MapiValue::MapiInt(val) => monet_escape(val.to_owned().to_string()),
        MapiValue::MapiBool(val) => monet_bool(val.to_owned() as bool),
    }
}

fn monet_escape(data: String) -> String {
    let old = data.replace("\\", "\\\\");

    format!("'{}'", old.replace("\'", "\\\'"))
}

fn monet_bool(data: bool) -> String {
    vec!["'false'", "'true'"][data as usize].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bools_are_escaped_correctly() {
        let _true = true;
        let _false = false;
        let should_be_true = "'true'";
        let should_be_false = "'false'";
    
        let is_this_true: String = monet_bool(_true);
        let is_this_false: String = monet_bool(_false);

        assert_eq!(is_this_true, should_be_true);
        assert_eq!(is_this_false, should_be_false);
    }
    
    #[test]
    fn parameters_are_applied_correctly() {
        let query = "SELECT * FROM demo WHERE {} = 1";
        let target = "SELECT * FROM demo WHERE 'id' = 1".to_string();
    
        let out: String = match apply_parameters(query, vec![MapiValue::MapiString("id")]) {
            Ok(e) => e,
            Err(_) => return assert!(false) 
        };

        assert_eq!(out, target);
    }

    #[test]
    fn multiple_parameters_are_applied_correctly() {
        let query = "SELECT * FROM demo WHERE {} = {}";
        let target = "SELECT * FROM demo WHERE 'id' = '1'".to_string();
    
        let out: String = match apply_parameters(query, vec![MapiValue::MapiString("id"), MapiValue::MapiInt(1)]) {
            Ok(e) => e,
            Err(_) => return assert!(false) 
        };

        assert_eq!(out, target);
    }

    #[test]
    fn bool_parameters_are_applied_correctly() {
        let query = "SELECT * FROM demo WHERE {} = {}";
        let target = "SELECT * FROM demo WHERE 'bogus' = 'true'".to_string();
        let target_false = "SELECT * FROM demo WHERE 'bogus' = 'false'".to_string();
    
        let out: String = match apply_parameters(query, vec![MapiValue::MapiString("bogus"), MapiValue::MapiBool(true)]) {
            Ok(e) => e,
            Err(_) => return assert!(false) 
        };

        let out_false: String = match apply_parameters(query, vec![MapiValue::MapiString("bogus"), MapiValue::MapiBool(false)]) {
            Ok(e) => e,
            Err(_) => return assert!(false) 
        };

        assert_eq!(out, target);
        assert_eq!(out_false, target_false);
    }

    #[test]
    fn parameters_cannot_contain_sql_injection_attacks() {
        let query = "SELECT * FROM demo WHERE {} = {}";
        let target = "SELECT * FROM demo WHERE 'id' = '1;COMMIT;DROP TABLE demo'".to_string();
    
        let out: String = match apply_parameters(query, vec![MapiValue::MapiString("id"), 
            MapiValue::MapiString("1;COMMIT;DROP TABLE demo")]) 
        {
            Ok(e) => e,
            Err(_) => return assert!(false) 
        };

        assert_eq!(out, target);
    }
}