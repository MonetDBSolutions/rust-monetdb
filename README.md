# monetdb-rust
A native Rust driver for MonetDB.

```toml
[dependencies]
monetdb-rust = "0.1.0"
```

# Overview

To be done.

## Example
```rust
import monetizer;
let mut c = Connection::connect("mapi://localhost:50000/demo").unwrap();

let mut params: Vec<monetizer::SQLParameter>;
params.push(monetizer::to_sqlparameter(3));
params.push(monetizer::to_sqlparameter(4));

let res = c.execute("INSERT INTO foo VALUES ({}), ({})", params).unwrap();
```


