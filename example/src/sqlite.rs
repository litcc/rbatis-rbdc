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
    let v = conn.get_values("select * from version", vec![]).await?;
    println!("{:?}", v);
    Ok(())
}
