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
        monetdb.execute("CREATE TABLE foo (i int, x int)", vec![])?;
        let result = monetdb.execute("INSERT INTO foo VALUES (1, 2), (2, 3)", vec![])?;

        assert_eq!(result, 2);
        let result = monetdb.execute(
            "INSERT INTO foo VALUES ({}, {}), ({}, {})",
            vec![to_sqlparameter(1), to_sqlparameter(2), to_sqlparameter(2), to_sqlparameter(4)],
        )?;

        assert_eq!(result, 2);

        Ok(())
    }

    #[test]
    fn query_row_test() -> Result<(), MonetDBError> {
        let mut monetdb = Connection::connect("mapi://localhost:50000/demo")?;
        monetdb.execute("DROP TABLE IF EXISTS foo2", vec![])?;
        monetdb.execute("CREATE TABLE foo2 (i int, x int)", vec![])?;
        let result = monetdb.execute("INSERT INTO foo2 VALUES (1, 2), (2, 3)", vec![])?;

        let result = monetdb.query(
            "SELECT * FROM foo2 WHERE i = ({})",
            vec![to_sqlparameter(2)],
        )?;

        assert_eq!(result.len(), 2); 
        assert_eq!(result, vec!["2", "3"]); 
    

        Ok(())
    }

}
