#[cfg(test)]
#[cfg(feature = "integration")]
mod tests {
    use crate::connection::Connection;

    use crate::monetizer::to_sqlparameter;
    use mapi::errors::MonetDBError;

    #[test]
    fn simple_connection_test() -> Result<(), MonetDBError> {
        let _monetdb = Connection::connect("mapi://localhost:50000/demo")?;
        Ok(())
    }

    #[test]
    fn create_insert_select_test() -> Result<(), MonetDBError> {
        let mut monetdb = Connection::connect("mapi://localhost:50000/demo")?;
        monetdb.execute("DROP TABLE IF EXISTS foo", vec![])?;
        monetdb.execute("CREATE TABLE foo (i int)", vec![])?;
        let result = monetdb.execute("INSERT INTO foo VALUES (1), (2)", vec![])?;

        assert_eq!(result, 2);
        let result = monetdb.execute(
            "INSERT INTO foo VALUES ({}), ({})",
            vec![to_sqlparameter(1), to_sqlparameter(2)],
        )?;
        assert_eq!(result, 2);
        let result = monetdb.execute("SELECT * FROM foo", vec![])?;
        assert_eq!(result, 0); // ! Not correct. The execute function needs work.

        let result = monetdb.query("SELECT * FROM foo")?;
        assert_eq!(result.len(), 4); // ! Not correct. The execute function needs work.
        assert_eq!(result, vec!["1", "2", "1", "2"]); // ! Not correct. The execute function needs work.

        Ok(())
    }

}
