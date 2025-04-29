use crate::db::Connection;
use crate::pool::manager::ConnectionManager;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// ConnectionGuard is a wrapper for a database connection make sure auto_close.
pub struct ConnectionGuard {
    pub conn: Option<Box<dyn Connection>>,
    pub manager_proxy: ConnectionManager,
    pub auto_close: Option<Duration>,
}

impl Debug for ConnectionGuard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionBox")
            .field("manager_proxy", &self.manager_proxy)
            .field("auto_close", &self.auto_close)
            .finish()
    }
}

unsafe impl Sync for ConnectionGuard {}

impl Deref for ConnectionGuard {
    type Target = Box<dyn Connection>;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().unwrap()
    }
}

impl DerefMut for ConnectionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().unwrap()
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Some(auto_close) = self.auto_close {
            if let Some(mut conn) = self.conn.take() {
                self.manager_proxy.spawn_task(async move {
                    let _ = tokio::time::timeout(auto_close, conn.close()).await;
                });
            }
        }
    }
}
