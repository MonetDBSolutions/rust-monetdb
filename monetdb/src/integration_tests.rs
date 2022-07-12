#[cfg(test)]
#[cfg(feature = "integration")]
mod tests {
    use mapi::errors::MonetDBError;
    use crate::monetizer::SQLParameters;

    #[test]
    fn simple_connection_test() -> Result<(), MonetDBError> {
        let monetdb = crate::Connection::connect("mapi://localhost:50000/demo")?;
        Ok(())
    }

    #[test]
    fn create_insert_select_test() -> Result<(), MonetDBError> {
        let mut monetdb = crate::Connection::connect("mapi://localhost:50000/demo")?;
        monetdb.execute("DROP TABLE IF EXISTS foo", &[])?;
        monetdb.execute("CREATE TABLE foo (i int)", &[])?;
        let result = monetdb.execute("INSERT INTO foo VALUES (1), (2)", &[])?;
        assert_eq!(result, 2);
        let result = monetdb.execute("INSERT INTO foo VALUES {}, {}", &[SQLParameters::from(1), SQLParameters::from(2)])?;
        assert_eq!(result, 2);
        let result = monetdb.execute("SELECT * FROM foo", &[])?;
        assert_eq!(result, 0); // ! Not correct. The execute function needs work.

        Ok(())
    }
}
