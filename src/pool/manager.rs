use crate::db::{ConnectOptions, Driver};
use crate::pool::guard::ConnectionGuard;
use crate::Error;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ConnectionManager {
    pub driver: Arc<Box<dyn Driver>>,
    pub option: Arc<Box<dyn ConnectOptions>>,
}

impl ConnectionManager {
    /// spawn task on runtime
    pub fn spawn_task<T>(&self, task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(task);
        }
    }

    pub fn new<D: Driver + 'static>(driver: D, url: &str) -> Result<Self, Error> {
        let mut option = driver.default_option();
        option.set_uri(url)?;
        Ok(Self {
            driver: Arc::new(Box::new(driver)),
            option: Arc::new(option),
        })
    }
    pub fn new_option<D: Driver + 'static, Option: ConnectOptions>(driver: D, option: Option) -> Self {
        Self {
            driver: Arc::new(Box::new(driver)),
            option: Arc::new(Box::new(option)),
        }
    }

    pub fn new_opt_box(driver: Box<dyn Driver>, option: Box<dyn ConnectOptions>) -> Self {
        Self {
            driver: Arc::new(driver),
            option: Arc::new(option),
        }
    }

    pub fn new_arc(driver: Arc<Box<dyn Driver>>, option: Arc<Box<dyn ConnectOptions>>) -> Self {
        Self {
            driver: driver,
            option: option,
        }
    }

    pub fn driver_type(&self) -> &str {
        self.driver.name()
    }

    pub async fn connect(&self) -> Result<ConnectionGuard, Error> {
        Ok(ConnectionGuard {
            conn: Some(self.driver.connect_opt(self.option.deref().deref()).await?),
            manager_proxy: self.clone(),
            auto_close: Some(Duration::from_secs(10)),
        })
    }

    pub async fn check(&self, conn: &mut ConnectionGuard) -> Result<(), Error> {
        match conn.ping().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
