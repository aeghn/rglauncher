use std::sync::Arc;
use anyhow::anyhow;
use rusqlite::{Connection, params};
use crate::plugins::{Plugin, PluginResult};
use core::result::Result;
use tracing::info;

pub struct HistoryPlugin {
    connection: Option<Connection>,
}


impl HistoryPlugin {
    pub fn new(path: &str) -> Self {
        info!("Creating History Plugin, path: {}", path);

        match Connection::open(path) {
            Ok(connection) => HistoryPlugin {
                connection: Some(connection),
            },
            Err(err) => HistoryPlugin { connection: None },
        }
    }

    pub fn get_only_ids(&self) -> anyhow::Result<Vec<String>> {
        if let Some(conn) = self.connection.as_ref() {
            let mut sql = conn.prepare("select id from result_history limit 30 order by update_time desc")?;
            let result : Vec<String> = sql.query_map([], |row| {
                row.get(0)
                })?.collect::<Result<Vec<String>, rusqlite::Error>>()?;

            return Ok(result)
        }

        Err(anyhow!("unable to read from connection"))
    }

    pub fn get_results(&self, input: &str) -> anyhow::Result<Vec<Arc<dyn PluginResult>>> {
        if let Some(conn) = self.connection.as_ref() {
            let mut sql = conn.prepare("select id, type, content from result_history where content like ? limit 30 order by update_time desc")?;
            let result = sql.query_map(params![input], |row| {
                let result_type: String = row.get(1)?;
                let result_str: String = row.get(2)?;

                Ok(Self::get_result(result_type.as_str(), result_str.as_str()).unwrap())
            })?.collect::<Result<Vec<Arc<dyn PluginResult>>, rusqlite::Error>>()?;

            return Ok(result)
        }

        Err(anyhow!("unable to read from connection"))
    }

    fn get_result(result_type: &str, result_str: &str) -> Result<Arc<dyn PluginResult>, serde_json::Error> {
        serde_json::from_str::<Box<dyn PluginResult>>(result_str).map(|r| Arc::from(r))
    }

    pub fn update_or_insert(&self, result: Arc<dyn PluginResult>) -> anyhow::Result<()> {
        let result_str = serde_json::to_string(result.as_ref())?;
        let conn = self.connection.as_ref().ok_or(anyhow!("There is no connection"))?;
        let mut stmt = conn.prepare("insert or replace into result_history \
        (id, type, content, update_time) values (?, ?, ?, datetime(?, 'unixepoch'))")?;

        stmt.insert(params![result.get_id(), result.get_type_id(), result_str,
            chrono::Utc::now().timestamp()]).expect("TODO: panic message");

        Ok(())
    }

    pub fn try_create_table(&self) -> anyhow::Result<()> {
        let conn = self.connection.as_ref().ok_or(anyhow!("There is no connection"))?;
        conn.prepare("
CREATE TABLE IF NOT EXISTS result_history (
id text primary key,
type text,
content TEXT,
update_time TIMESTAMP
)")?.execute([])?;


        Ok(())
    }
}
