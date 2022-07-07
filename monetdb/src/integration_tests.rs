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
}