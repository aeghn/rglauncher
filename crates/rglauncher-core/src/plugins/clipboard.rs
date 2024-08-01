use anyhow::{anyhow, Error};
use arboard::Clipboard;
use async_sqlite::rusqlite;
use chrono::{DateTime, Utc};

use crate::config::DbConfig;
use crate::db::Db;
use crate::plugins::{PluginItemTrait, PluginTrait};
use crate::userinput::UserInput;
use crate::util::scoreutils;
use tracing::info;

pub const TYPE_NAME: &str = "clipboard";

#[derive(Clone)]
pub enum ClipMsg {}

#[derive(Debug, Clone)]
pub struct ClipItem {
    pub content: String,
    score: i32,
    pub mime: String,
    pub insert_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub count: i64,
    pub id: String,
}

impl PluginItemTrait for ClipItem {
    fn get_score(&self) -> i32 {
        scoreutils::low(self.score as i64)
    }

    fn on_activate(&self) {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(self.content.as_str()).unwrap();
    }

    fn get_type(&self) -> &'static str {
        &TYPE_NAME
    }

    fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

pub struct ClipboardPlugin {
    connection: Db,
}

impl ClipboardPlugin {
    pub async fn new(config: Option<&DbConfig>) -> anyhow::Result<Self> {
        match config.map(|e| e.db_path.as_str()) {
            Some(path) => {
                info!("Creating Clip Plugin, config: {:?}", path);

                match Db::new(path).await {
                    Ok(db) => Ok(Self { connection: db }),
                    Err(err) => {
                        anyhow::bail!("missing database config {}", err)
                    }
                }
            }
            None => anyhow::bail!("missing database config"),
        }
    }
}

impl PluginTrait for ClipboardPlugin {
    async fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<ClipItem>> {
        let user_input = user_input.clone();
        if user_input.input.is_empty() {
            return Err(anyhow!("empty input"));
        }

        let client = self.connection.client.clone();

        let result = client
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT content0, mimes, insert_time, update_time, count \
            from clipboard where content0 like ? order by UPDATE_TIME desc limit 100",
                )?;

                let result = stmt
                    .query_map([format!("%{}%", user_input.input.as_str())], |row| {
                        Ok(ClipItem {
                            content: row.get(0)?,
                            score: 0,
                            mime: row.get(1)?,
                            insert_time: row.get(2)?,
                            update_time: row.get(3)?,
                            count: row.get(4)?,
                            id: format!("{:x}", md5::compute(&(row.get::<usize, String>(0)?))),
                        })
                    })?
                    .collect::<Result<Vec<ClipItem>, rusqlite::Error>>()?;

                Ok(result)
            })
            .await;

        Ok(result?)
    }

    fn get_type(&self) -> &'static str {
        &TYPE_NAME
    }

    type Item = ClipItem;

    type Msg = ClipMsg;
}
