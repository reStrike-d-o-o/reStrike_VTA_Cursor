use std::{path::Path, time::Duration};

use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement,
};

use super::DATABASE_FILE;

/// Type alias used throughout the application whenever a SeaORM connection is required.
pub type SeaOrmConnection = DatabaseConnection;

/// Establish a SeaORM SQLite connection using the canonical database file within `base_dir`.
pub async fn connect_default(base_dir: impl AsRef<Path>) -> Result<SeaOrmConnection, DbErr> {
    let db_path = base_dir.as_ref().join(DATABASE_FILE);
    connect(db_path).await
}

/// Establish a SeaORM SQLite connection against an explicit database file.
pub async fn connect(path: impl AsRef<Path>) -> Result<SeaOrmConnection, DbErr> {
    let path = path.as_ref();
    let uri = format!(
        "sqlite://{}?mode=rwc",
        path.to_string_lossy().replace('\\', "/")
    );

    let mut options = ConnectOptions::new(uri);
    options
        .max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(5))
        .sqlx_logging(false);

    let connection = Database::connect(options).await?;

    // Ensure foreign key enforcement stays active for the session.
    connection
        .execute(Statement::from_string(
            DbBackend::Sqlite,
            "PRAGMA foreign_keys = ON".to_string(),
        ))
        .await?;

    Ok(connection)
}
