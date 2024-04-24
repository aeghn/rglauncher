use anyhow::{anyhow, Error};
use arboard::Clipboard;
use chrono::{DateTime, Utc};

use crate::config::DbConfig;
use crate::plugins::history::HistoryItem;
use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use crate::util::score_utils;
use rusqlite::Connection;
use tracing::info;

pub mod watcher;

pub const TYPE_ID: &str = "clipboard";

#[derive(Clone)]
pub enum ClipMsg {}

#[derive(Debug)]
pub struct ClipResult {
    pub content: String,
    score: i32,
    pub mime: String,
    pub insert_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub count: i64,
    pub id: String,
}

impl PluginResult for ClipResult {
    fn score(&self) -> i32 {
        score_utils::low(self.score as i64)
    }

    fn icon_name(&self) -> &str {
        "xclipboard"
    }

    fn name(&self) -> &str {
        self.content.as_str()
    }

    fn extra(&self) -> Option<&str> {
        None
    }

    fn on_enter(&self) {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(self.content.as_str()).unwrap();
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

pub struct ClipboardPlugin {
    connection: Option<Connection>,
}

impl ClipboardPlugin {
    pub fn new(config: Option<&DbConfig>) -> anyhow::Result<Self> {
        match config.map(|e| e.db_path.as_str()) {
            Some(path) => {
                info!("Creating Clip Plugin, config: {:?}", path);

                match Connection::open(path) {
                    Ok(connection) => {
                        let plugin = ClipboardPlugin {
                            connection: Some(connection),
                        };

                        Ok(plugin)
                    }
                    Err(err) => Err(err.into()),
                }
            }
            None => anyhow::bail!("missing database config"),
        }
    }

    fn update(&mut self) {
        info!("update TODO");
    }
}

impl Plugin<ClipResult, ClipMsg> for ClipboardPlugin {
    fn handle_msg(&mut self, _msg: ClipMsg) {
        todo!()
    }

    fn refresh_content(&mut self) {
        self.update()
    }

    fn handle_input(
        &self,
        user_input: &UserInput,
        _history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<ClipResult>> {
        if let Some(conn) = &self.connection {
            let mut clip_results = Vec::new();

            let like_stmt = "
SELECT content, mime, insert_time, update_time, count, id
FROM clipboard
WHERE content LIKE ?
ORDER BY update_time DESC
LIMIT 100
";

            let all_stmt = "
SELECT content, mime, insert_time, update_time, count, id
FROM clipboard
ORDER BY update_time DESC
LIMIT 100
";

            let (stmt_pharse, query) = if user_input.input.is_empty() {
                (all_stmt, None)
            } else {
                (like_stmt, Some(user_input.input.clone()))
            };

            let mut stmt = conn.prepare(stmt_pharse)?;

            let mut rows = match query {
                Some(q) => stmt.query([&q])?,
                None => stmt.query([])?,
            };

            while let Some(row) = rows.next()? {
                let content: String = row.get(0)?;
                let mime: String = row.get(1)?;
                let insert_time: DateTime<Utc> = row.get(2)?;
                let update_time: DateTime<Utc> = row.get(3)?;
                let count: i64 = row.get(4)?;
                let id: String = row.get(5)?;

                let clip_result = ClipResult {
                    content,
                    score: 0,
                    mime,
                    insert_time,
                    update_time,
                    count,
                    id,
                };
                clip_results.push(clip_result);
            }

            Ok(clip_results)
        } else {
            Err(Error::msg("unable to find connection"))
        }
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}
