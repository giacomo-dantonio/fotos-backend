use sqlx::{migrate::{MigrateDatabase, Migrate}, Pool, Database};

pub async fn ensure_db<A>(url: &str) -> anyhow::Result<bool>
    where A: MigrateDatabase
{
    if !A::database_exists(url).await.unwrap_or(false) {
        A::create_database(url).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn migrate<A, B>(pool: &Pool<A>) -> anyhow::Result<()>
    where
        A: Database<Connection = B>,
        B: Migrate
{
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

