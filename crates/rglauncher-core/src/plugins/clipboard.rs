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
                    Ok(connection) => Ok(ClipboardPlugin {
                        connection: Some(connection),
                    }),
                    Err(err) => Err(err.into()),
                }
            }
            None => anyhow::bail!("missing database config"),
        }
    }
}

impl Plugin<ClipResult, ClipMsg> for ClipboardPlugin {
    fn handle_msg(&mut self, _msg: ClipMsg) {
        todo!()
    }

    fn refresh_content(&mut self) {}

    fn handle_input(
        &self,
        user_input: &UserInput,
        _history: Option<Vec<HistoryItem>>,
    ) -> anyhow::Result<Vec<ClipResult>> {
        if user_input.input.is_empty() {
            return Err(anyhow!("empty input"));
        }

        if let Some(conn) = self.connection.as_ref() {
            let mut stmt = conn.prepare(
                "SELECT content0, mimes, insert_time, update_time, count \
        from clipboard where content0 like ? order by UPDATE_TIME desc limit 100",
            )?;

            let result = stmt
                .query_map([format!("%{}%", user_input.input.as_str())], |row| {
                    Ok(ClipResult {
                        content: row.get(0)?,
                        score: 0,
                        mime: row.get(1)?,
                        insert_time: row.get(2)?,
                        update_time: row.get(3)?,
                        count: row.get(4)?,
                        id: format!("{:x}", md5::compute(&(row.get::<usize, String>(0)?))),
                    })
                })?
                .collect::<Result<Vec<ClipResult>, rusqlite::Error>>()?;

            Ok(result)
        } else {
            Err(Error::msg("unable to find connection"))
        }
    }

    fn get_type_id(&self) -> &'static str {
        &TYPE_ID
    }
}
