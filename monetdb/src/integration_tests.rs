#[cfg(test)]
#[cfg(feature = "integration")]
mod tests {
    #[test]
    fn we_can_connect() {
        let monetdb = crate::Connection::connect("mapi://localhost:50000/demo");
        match monetdb {
            Ok(_) => assert!(true),
            Err(e) => {
                eprintln!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn we_can_create_a_table() {
        let mut monetdb = crate::Connection::connect("mapi://localhost:50000/demo").unwrap();
        let result = monetdb.execute("CREATE TABLE IF NOT EXISTS foo (i int)");
        match result {
            Ok(_) => assert!(true),
            Err(e) => {
                eprintln!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn we_can_insert_into_a_table() {
        let mut monetdb = crate::Connection::connect("mapi://localhost:50000/demo").unwrap();
        let result = monetdb.execute("INSERT INTO foo VALUES (1), (2)");
        match result {
            Ok(_) => assert!(true),
            Err(e) => {
                eprintln!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn we_can_select_from_a_table() {
        let mut monetdb = crate::Connection::connect("mapi://localhost:50000/demo").unwrap();
        let result = monetdb.execute("SELECT * FROM foo");
        match result {
            Ok(_) => assert!(true),
            Err(e) => {
                eprintln!("{}", e);
                assert!(false);
            }
        }
    }
}