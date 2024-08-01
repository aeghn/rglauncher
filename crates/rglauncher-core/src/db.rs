use std::path::Path;

use async_sqlite::{Client, ClientBuilder, JournalMode};

pub struct Db {
    pub client: Client,
}

impl Db {
    pub async fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let client = ClientBuilder::new()
            .path(path)
            .journal_mode(JournalMode::Wal)
            .open()
            .await?;

        let db = Self { client };

        Ok(db)
    }
}
