use std::{sync::Arc, env};

use axum::extract::State;
use sqlx::{SqlitePool, Sqlite};
use uuid::Uuid;

use crate::{AppState, AppConf, infrastructure};

static DB_URL: &str = "sqlite://test.db";

pub async fn setup(pool: &SqlitePool) {
    infrastructure::ensure_db::<Sqlite>(DB_URL).await
        .expect("Unable to create test database");
    infrastructure::migrate(pool).await
        .expect("Database migration failed")
}

pub async fn make_state() -> State<Arc<AppState>> {
    let root = env::current_dir()
        .unwrap()
        .join("data");
    let root = root.to_str().unwrap();
    let conf = AppConf {
        root: root.to_string(),
        connection: "0.0.0.0:3000".to_string(),
        max_level: "DEBUG".to_string()
    };

    let pool = SqlitePool::connect(DB_URL)
        .await
        .unwrap();
    let state = AppState {
        conf,
        pool
    };

    State(Arc::new(state))
}

pub async fn insert_tags(names: impl Iterator<Item=&str>, pool: &SqlitePool) {
    // FIXME: this has concurrency issues when the tests run in parallel
    sqlx::query("DELETE FROM tags WHERE 1=1")
        .execute(pool)
        .await
        .expect("Unable to delete old tags");

    for name in names {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO tags (id, tagname) VALUES ($1, $2)")
            .bind(id.to_string())
            .bind(name)
            .execute(pool)
            .await
            .expect("Unable to insert tag");
    }
}
