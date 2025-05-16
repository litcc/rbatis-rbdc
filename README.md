RBDC driver abstract

* `rbdc` is safe code(`#[forbid(unsafe_code)]`)
* an database driver abstract for `rbatis`
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
use rbdc::pool::ConnManager;
use rbdc::pool::Pool;
use rbdc_pool_fast::FastPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = FastPool::new(ConnManager::new(SqliteDriver {}, "sqlite://target/test.db")?)?;
    let mut conn = pool.get().await?;
    // select
    let v = conn.get_values("select * from sqlite_master", vec![]).await?;
    println!("{}", rbs::Value::Array(v));
    // update/delete
    let r = conn.exec("update table set name='a' where id = 1", vec![]).await?;
    println!("{}", r);
    Ok(())
}

```


### FAQ

#### How should I implement a driver for databases with blocking APIs?

For database drivers with blocking APIs, follow the pattern in `rbdc-sqlite` using the `flume` channel library:

```rust
// Key components:
// 1. Dedicated worker thread per connection
// 2. Command channels for communication

pub struct YourConnection {
    worker: ConnectionWorker,
}

struct ConnectionWorker {
    command_tx: flume::Sender<Command>,
}

enum Command {
    Execute { /* ... */ },
    Query { /* ... */ },
}
```

Benefits:
- Prevents blocking the async runtime
- Provides thread safety
- Maintains a clean async interface

#### Why does `Connection` require both `Send` and `Sync`?

`Connection: Send + Sync` is required because:

1. **Thread Safety**: Connections may be shared across tasks on different threads when using Tokio
2. **Pool Implementation**: Connection pools need thread-safe access to connections

When implementing for non-thread-safe databases:

```rust
// SAFETY: YourConnection is thread-safe because:
// 1. Database operations run on a dedicated worker thread
// 2. Communication uses thread-safe channels
unsafe impl Sync for YourConnection {}
```

Improper implementation can cause data races and undefined behavior.