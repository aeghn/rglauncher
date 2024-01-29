use crate::userinput::UserInput;
use crate::{plugins::PluginResult, util::score_utils};
use core::result::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use lru_cache::LruCache;
use rusqlite::{params, Connection};
use std::{cell::RefCell, sync::Arc};
use tracing::info;

#[derive(Clone, Debug)]
pub struct HistoryItem {
    pub plugin_type: String,
    pub id: String,
    pub result_name: String,
    pub score: i32,
}

pub struct HistoryPlugin {
    connection: Connection,
    memory_cache: RefCell<LruCache<String, HistoryItem>>,
    matcher: SkimMatcherV2,
}

impl HistoryPlugin {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        info!("Creating History Plugin, path: {}", path);

        let connection = Connection::open(path)?;
        Self::try_create_table(&connection).unwrap();

        let mut memory_cache = LruCache::new(268);
        Self::get_histories_from_db(&connection)?
            .into_iter()
            .for_each(|h| {
                memory_cache.insert(h.id.clone(), h);
            });

        let matcher = SkimMatcherV2::default();

        Ok(HistoryPlugin {
            connection,
            memory_cache: RefCell::new(memory_cache),
            matcher,
        })
    }

    pub fn get_histories(&self, user_input: &UserInput) -> Vec<HistoryItem> {
        self.memory_cache
            .borrow()
            .iter()
            .filter(|(_, v)| {
                if user_input.input.is_empty() {
                    true
                } else {
                    self.matcher
                        .fuzzy_match(v.result_name.as_str(), user_input.input.as_str())
                        .unwrap_or(0)
                        > 0
                }
            })
            .map(|h| {h.1})
            .enumerate()
            .map(|(i, e)| {
                let mut h = e.clone();
                h.score = score_utils::highest(i as i16);
                h
            })
            .collect()
    }

    fn get_histories_from_db(conn: &Connection) -> anyhow::Result<Vec<HistoryItem>> {
        let mut stmt = conn.prepare("select id, type, name from result_history order by update_time desc limit 268")?;

        let result = stmt
            .query_map(params![], |row| {
                let id: String = row.get("id")?;
                let plugin_type: String = row.get("type")?;
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
        let mut stmt = self.connection.prepare(
            "insert or replace into result_history \
        (id, type, name, description, update_time) values (?, ?, ?, ?, datetime(?, 'unixepoch'))",
        )?;

        stmt.insert(params![
            result.get_id(),
            result.get_type_id(),
            result.name(),
            result.extra(),
            chrono::Utc::now().timestamp()
        ])
        .expect("Unable to insert history");

        self.memory_cache.borrow_mut().insert(
            result.get_id().to_string(),
            HistoryItem {
                plugin_type: result.get_type_id().to_string(),
                id: result.get_id().to_string(),
                result_name: result.name().to_string(),
                score: 0,
            },
        );
        

        Ok(())
    }

    fn try_create_table(conn: &Connection) -> anyhow::Result<()> {
        conn.prepare(
            "CREATE TABLE IF NOT EXISTS result_history (
id TEXT,
type TEXT,
name TEXT,
description TEXT,
update_time TIMESTAMP,
PRIMARY KEY (id, type)
)",
        )?
        .execute([])?;

        Ok(())
    }
}
