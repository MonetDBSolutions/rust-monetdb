#[cfg(test)]
#[cfg(feature = "integration")]
mod tests {
    use mapi::errors::MonetDBError;
    #[test]
    fn simple_connection_test() -> Result<(), MonetDBError> {
        let monetdb = crate::Connection::connect("mapi://localhost:50000/demo")?;
        Ok(())
    }

    #[test]
    fn create_insert_select_test() -> Result<(), MonetDBError> {
        let mut monetdb = crate::Connection::connect("mapi://localhost:50000/demo")?;
        monetdb.execute("DROP TABLE IF EXISTS foo")?;
        monetdb.execute("CREATE TABLE foo (i int)")?;
        let result = monetdb.execute("INSERT INTO foo VALUES (1), (2)")?;
        assert_eq!(result, 2);
        let result = monetdb.query("SELECT * FROM foo")?;
        assert_eq!(result.len(), 2); // ! Not correct. The execute function needs work.
        assert_eq!(result, vec!["1", "2"]); // ! Not correct. The execute function needs work.

        Ok(())
    }
}
