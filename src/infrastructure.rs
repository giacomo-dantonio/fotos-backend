use sqlx::{Sqlite, migrate::MigrateDatabase};

pub async fn ensure_db(url: &str) -> anyhow::Result<bool>{
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        Sqlite::create_database(url).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?;
