# monetdb-rust
A native Rust driver for MonetDB.

```toml
[dependencies]
monetdb-rust = "0.1.0"
```

# Overview

## Example
```rust
let mut c = Connection::connect("mapi://localhost:50000/demo").unwrap();

let res = c.execute("INSERT INTO foo VALUES {}, {}", &[monetizer::SQLParameters::from(3), monetizer::SQLParameters::from(4)]).unwrap();
```

To be done. 