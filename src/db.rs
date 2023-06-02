use sea_orm::{Database, DatabaseConnection};
use std::sync::OnceLock;

static DB: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init(db_url: &str) {
    let db = Database::connect(db_url).await.unwrap();
    DB.set(db).unwrap();
}

/// database access handler
pub fn hdr<'a>() -> &'a DatabaseConnection {
    DB.get().unwrap()
}
