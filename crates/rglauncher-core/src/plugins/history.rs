use crate::config::DbConfig;
use crate::plugins::PluginResult;

use core::result::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use rusqlite::{params, Connection};
use std::{
    any::Any,
    sync::{Arc, RwLock},
};
use tracing::info;

#[derive(Clone, Debug)]
pub struct HistoryItem {
    pub plugin_type: String,
    pub id: String,
    pub result_name: String,
    pub score: i32,
}

pub struct HistoryPlugin {
    connection: Option<Connection>,
    memory_cache: Arc<RwLock<Vec<HistoryItem>>>,
}

impl HistoryPlugin {
    pub fn new(config: Option<&DbConfig>) -> Self {
        info!("Creating History Plugin, path: {:?}", config);
        let connection = match config.map(|c| c.db_path.as_str()) {
            Some(path) => {
                let connection = Connection::open(path);
                if let Ok(conn) = connection {
                    Self::try_create_table(&conn).unwrap();
                    Some(conn)
                } else {
                    None
                }
            }
            None => None,
        };

        let memory_cache = match connection.as_ref() {
            Some(conn) => Self::get_histories_from_db(&conn).map_or(vec![], |e| e),
            None => {
                vec![]
            }
        };

        HistoryPlugin {
            connection,
            memory_cache: Arc::new(RwLock::new(memory_cache)),
        }
    }

    pub fn get_cache(&self) -> Arc<RwLock<Vec<HistoryItem>>> {
        self.memory_cache.clone()
    }

    fn get_histories_from_db(conn: &Connection) -> anyhow::Result<Vec<HistoryItem>> {
        let mut stmt = conn.prepare(
            "select id, plugin_type, name from result_history order by update_time desc limit 268",
        )?;

        let result = stmt
            .query_map(params![], |row| {
                let id: String = row.get("id")?;
                let plugin_type: String = row.get("plugin_type")?;
                let result_name: String = row.get("name")?;

                Ok(HistoryItem {
                    plugin_type,
                    id,
                    result_name,
                    score: 0,
                })
            })?
            .collect::<Result<Vec<HistoryItem>, rusqlite::Error>>()?;

        Ok(result.into_iter().rev().collect())
    }

    pub fn update_or_insert(&self, result: Arc<dyn PluginResult>) -> anyhow::Result<()> {
        if let Some(conn) = self.connection.as_ref() {
            let mut stmt = conn.prepare(
                "insert or replace into result_history \
            (id, plugin_type, name, content, update_time) values (?, ?, ?, ?, datetime(?, 'unixepoch'))",
            )?;

            stmt.insert(params![
                result.get_id(),
                result.get_type_id(),
                result.name(),
                result.extra(),
                chrono::Utc::now().timestamp()
            ])?;
        }

        if let Ok(mut guard) = self.memory_cache.write() {
            let vec: &mut Vec<HistoryItem> = guard.as_mut();
            vec.retain(|e| e.id != result.get_id() || e.plugin_type != result.get_type_id());
            vec.truncate(100);
            vec.push(HistoryItem {
                plugin_type: result.get_type_id().to_string(),
                id: result.get_id().to_string(),
                result_name: result.name().to_string(),
                score: 0,
            });
        }

        Ok(())
    }

    fn try_create_table(conn: &Connection) -> anyhow::Result<()> {
        conn.prepare(
            "CREATE TABLE IF NOT EXISTS result_history (
id TEXT,
plugin_type TEXT,
name TEXT,
content TEXT,
update_time TIMESTAMP,
PRIMARY KEY (id, plugin_type)
)",
        )?
        .execute([])?;

        Ok(())
    }
}
