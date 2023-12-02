![](https://github.com/wsfy15/rust-lib-template/workflows/build/badge.svg)

# sqlx-database-tester-wsf

sqlx tester for postgres, work with tokio only.

create one database at start of testing and destory after test ending.

## How to use it

```rust
let tdb = TestDB::new("localhost", 5432, "postgres", "123456", "./migrations");
let pool = tdb.get_pool().await;
// do something with pool

// when tdb get dropped, the database will be dropped
```

Have fun with this crate!

## License

This project is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

Copyright 2021 SF Wu
