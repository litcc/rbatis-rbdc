use futures_core::future::BoxFuture;
use futures_intrusive::sync::MutexGuard;
use futures_util::future;
use libsqlite3_sys::sqlite3;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::ptr::NonNull;

pub(crate) use handle::{ConnectionHandle, ConnectionHandleRaw};

use crate::connection::establish::EstablishParams;
use crate::connection::worker::ConnectionWorker;
use crate::statement::VirtualStatement;
use crate::SqliteConnectOptions;
use rbdc::error::Error;
use rbdc::StatementCache;

pub(crate) mod collation;
mod establish;
mod execute;
mod executor;
mod handle;

mod worker;
pub use worker::Command;

/// A connection to an open [Sqlite] database.
///
/// Because SQLite is an in-process database accessed by blocking API calls, rbdc uses a background
/// thread and communicates with it via channels to allow non-blocking access to the database.
///
/// Dropping this struct will signal the worker thread to quit and close the database, though
/// if an error occurs there is no way to pass it back to the user this way.
///
/// You can explicitly call [`.close()`][Self::close] to ensure the database is closed successfully
/// or get an error otherwise.
pub struct SqliteConnection {
    pub(crate) worker: ConnectionWorker,
    pub(crate) row_channel_size: usize,
}

// SAFETY: SqliteConnection is safe to share between threads because:
// 1. The `worker` field is marked as `pub(crate)`, preventing external users from directly accessing it
// 2. All SQLite operations are sent through thread-safe flume channels to a dedicated worker thread
// 3. The `worker.handle_raw` pointer (raw SQLite connection) is encapsulated within the crate
// 4. Any operation requiring direct access to the SQLite connection uses the `lock_handle()` method
//    which acquires a mutex lock, ensuring synchronized access
// 5. The only potentially unsafe method `as_raw_handle()` is marked as deprecated with warnings
//    about thread safety and should be marked as `unsafe` to properly indicate its risks
unsafe impl Sync for SqliteConnection {}

pub struct LockedSqliteHandle<'a> {
    pub(crate) guard: MutexGuard<'a, ConnectionState>,
}

pub struct ConnectionState {
    pub(crate) handle: ConnectionHandle,

    pub(crate) statements: Statements,
}

pub(crate) struct Statements {
    // cache of semi-persistent statements
    cached: StatementCache<VirtualStatement>,
    // most recent non-persistent statement
    temp: Option<VirtualStatement>,
}

impl SqliteConnection {
    pub(crate) async fn establish(options: &SqliteConnectOptions) -> Result<Self, Error> {
        let params = EstablishParams::from_options(options)?;
        let worker = ConnectionWorker::establish(params).await?;
        Ok(Self {
            worker,
            row_channel_size: options.row_channel_size,
        })
    }

    /// Returns the underlying sqlite3* connection handle.
    ///
    /// ### Note
    /// There is no synchronization using this method, beware that the background thread could
    /// be making SQLite API calls concurrent to use of this method.
    ///
    /// You probably want to use [`.lock_handle()`][Self::lock_handle] to ensure that the worker thread is not using
    /// the database concurrently.
    #[deprecated(note = "Unsynchronized access is unsafe. See documentation for details.")]
    pub unsafe fn as_raw_handle(&mut self) -> *mut sqlite3 {
        self.worker.handle_raw.as_ptr()
    }

    /// Apply a collation to the open database.
    ///
    /// See [`SqliteConnectOptions::collation()`] for details.
    ///
    /// ### Deprecated
    /// Due to the rearchitecting of the SQLite driver, this method cannot actually function
    /// synchronously and return the result directly from `sqlite3_create_collation_v2()`, so
    /// it instead sends a message to the worker create the collation asynchronously.
    /// If an error occurs it will simply be logged.
    ///
    /// Instead, you should specify the collation during the initial configuration with
    /// [`SqliteConnectOptions::collation()`]. Then, if the collation fails to apply it will
    /// return an error during the connection creation. When used with a [Pool][crate::pool::Pool],
    /// this also ensures that the collation is applied to all connections automatically.
    ///
    /// Or if necessary, you can call [`.lock_handle()`][Self::lock_handle]
    /// and create the collation directly with [`LockedSqliteHandle::create_collation()`].
    ///
    /// [`Error::from("WorkerCrashed")`] may still be returned if we could not communicate with the worker.
    ///
    /// Note that this may also block if the worker command channel is currently applying
    /// backpressure.
    #[deprecated(note = "Completes asynchronously. See documentation for details.")]
    pub fn create_collation(
        &mut self,
        name: &str,
        compare: impl Fn(&str, &str) -> Ordering + Send + Sync + 'static,
    ) -> Result<(), Error> {
        self.worker.create_collation(name, compare)
    }

    /// Lock the SQLite database handle out from the worker thread so direct SQLite API calls can
    /// be made safely.
    ///
    /// Returns an error if the worker thread crashed.
    pub async fn lock_handle(&mut self) -> Result<LockedSqliteHandle<'_>, Error> {
        let guard = self.worker.unlock_db().await?;

        Ok(LockedSqliteHandle { guard })
    }
}

impl Debug for SqliteConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteConnection")
            .field("row_channel_size", &self.row_channel_size)
            .field("cached_statements_size", &self.cached_statements_size())
            .finish()
    }
}

impl SqliteConnection {
    pub async fn do_close(&mut self) -> Result<(), Error> {
        // Drop the statement worker, which should
        // cover all references to the connection handle outside of the worker thread
        // Ensure the worker thread has terminated
        self.worker.shutdown().await
    }

    /// Ensure the background worker thread is alive and accepting commands.
    pub fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.worker.ping())
    }

    pub fn cached_statements_size(&self) -> usize {
        self.worker
            .shared
            .cached_statements_size
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.worker.clear_cache().await?;
            Ok(())
        })
    }

    #[doc(hidden)]
    pub fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite, FLUSH does effectively nothing...
        // Well, we could use this to ensure that the command channel has been cleared,
        // but it would only develop a backlog if a lot of queries are executed and then cancelled
        // partway through, and then this would only make that situation worse.
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    pub fn should_flush(&self) -> bool {
        false
    }
}

impl LockedSqliteHandle<'_> {
    /// Returns the underlying sqlite3* connection handle.
    ///
    /// As long as this `LockedSqliteHandle` exists, it is guaranteed that the background thread
    /// is not making FFI calls on this database handle or any of its statements.
    pub fn as_raw_handle(&mut self) -> NonNull<sqlite3> {
        self.guard.handle.as_non_null_ptr()
    }

    /// Apply a collation to the open database.
    ///
    /// See [`SqliteConnectOptions::collation()`] for details.
    pub fn create_collation(
        &mut self,
        name: &str,
        compare: impl Fn(&str, &str) -> Ordering + Send + Sync + 'static,
    ) -> Result<(), Error> {
        collation::create_collation(&mut self.guard.handle, name, compare)
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        // explicitly drop statements before the connection handle is dropped
        self.statements.clear();
    }
}

impl Statements {
    fn new(capacity: usize) -> Self {
        Statements {
            cached: StatementCache::new(capacity),
            temp: None,
        }
    }

    fn get(&mut self, query: &str, persistent: bool) -> Result<&mut VirtualStatement, Error> {
        if !persistent || !self.cached.is_enabled() {
            return Ok(self.temp.insert(VirtualStatement::new(query, false)?));
        }

        let exists = self.cached.contains_key(query);

        if !exists {
            let statement = VirtualStatement::new(query, true)?;
            self.cached.insert(query, statement);
        }
        //.unwrap() is safe
        let statement = self.cached.get_mut(query).unwrap();

        if exists {
            // as this statement has been executed before, we reset before continuing
            statement.reset()?;
        }

        Ok(statement)
    }

    fn len(&self) -> usize {
        self.cached.len()
    }

    fn clear(&mut self) {
        self.cached.clear();
        self.temp = None;
    }
}
