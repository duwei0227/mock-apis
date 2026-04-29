pub mod log_store;
pub mod mock_store;
pub mod port_store;
pub mod schema;

use crate::error::Result;
use tokio_rusqlite::Connection;

pub use log_store::SqliteLogStore;
pub use mock_store::SqliteMockStore;
pub use port_store::SqlitePortStore;

pub async fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path).await?;
    conn.call(|c| {
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        schema::run_migrations(c)?;
        Ok(())
    })
    .await?;
    Ok(conn)
}
