use anyhow::{anyhow, Error};
use arboard::Clipboard;
use chrono::{DateTime, Utc};

use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;
use crate::util::score_utils;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::info;

pub const TYPE_ID: &str = "clipboard";

pub enum ClipMsg {}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ClipResult {
    pub content: String,
    score: i32,
    pub mime: String,
    pub insert_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub count: i64,
    pub id: String,
}

#[typetag::serde]
impl PluginResult for ClipResult {
    fn score(&self) -> i32 {
        score_utils::low(self.score as i64)
    }

    fn sidebar_icon_name(&self) -> String {
        "xclipboard".to_string()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.insert_time.to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(crate::util::string_utils::truncate(self.content.as_str(), 200).to_string())
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
    pub fn new(path: &str) -> Self {
        info!("Creating Clip Plugin, path: {}", path);

        match Connection::open(path) {
            Ok(connection) => ClipboardPlugin {
                connection: Some(connection),
            },
            Err(err) => ClipboardPlugin { connection: None },
        }
    }
}

impl Plugin<ClipResult, ClipMsg> for ClipboardPlugin {
    fn handle_msg(&mut self, msg: ClipMsg) {
        todo!()
    }

    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<ClipResult>> {
        if user_input.input.is_empty() {
            return Err(anyhow!("empty input"))
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
