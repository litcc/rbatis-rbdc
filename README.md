RBDC driver abstract

* an database driver abstract for rbatis
* supported database drivers see [rbatis](https://github.com/rbatis/rbatis)

### how to define my driver to support rbdc driver?
just only impl this traits(6)
```rust
use rbdc::db::{Driver, MetaData, Row, Connection, ConnectOptions, Placeholder};

pub struct YourDriver{}
impl Driver for YourDriver{}

pub struct YourMetaData{}
impl MetaData for YourMetaData{}

pub struct YourRow{}
impl Row for YourRow{}

pub struct YourConnection{}
impl Connection for YourConnection{}

pub struct YourConnectOptions{}
impl ConnectOptions for YourConnectOptions{}

pub struct YourPlaceholder{}
impl Placeholder for YourPlaceholder{}
```

### how to use my driver?
* more [examples](example)
* for sqlite example
```rust
use rbdc_sqlite::SqliteDriver;
use rbdc::db::{Connection};
use rbdc::Error;
use rbdc::pool::conn_manager::ConnManager;
use rbdc::pool::Pool;
use rbdc_pool_fast::FastPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = FastPool::new(ConnManager::new(SqliteDriver {}, "sqlite://target/test.db")?)?;
    let mut conn = pool.get().await?;
    let v = conn.get_values("select * from sqlite_master", vec![]).await?;
    println!("{}", rbs::Value::Array(v));
    Ok(())
}

```
