use anyhow::Context;
use chin_tools::AResult;
use chrono::NaiveDateTime;
use rusqlite::{params, Connection};
use serde::{de::DeserializeOwned, Serialize};

use super::{PluginResult, PluginResultEnum};

#[derive(Clone)]
pub struct HistoryItem<V: Clone> {
    pub id: String,
    pub plugin_type: String,
    pub body: V,
    pub weight: f64,
    pub update_time: NaiveDateTime,
}

pub struct HistoryPlugin<'a> {
    conn: Option<&'a Connection>,
}

impl<'a> HistoryPlugin<'a> {
    pub fn new(conn: Option<&'a Connection>) -> Self {
        HistoryPlugin { conn }
    }

    pub fn get_id(pr: &PluginResultEnum) -> String {
        return pr.get_type_id().to_owned() + pr.get_id();
    }

    pub fn fetch_histories<V: Clone + DeserializeOwned>(&self) -> AResult<Vec<HistoryItem<V>>> {
        let mut stmt = self.conn
            .context("conn is none")?.prepare(
                "select id, plugin_type, body_json, weight, update_time from result_history order by update_time desc limit 268",
            )?;

        let result = stmt
            .query_map(params![], |row| {
                let id: String = row.get("id")?;
                let plugin_type: String = row.get("plugin_type")?;
                let body_json: String = row.get("body_json")?;
                let weight: f64 = row.get("weight")?;
                let update_time = row.get("update_time")?;

                let body: V = serde_json::from_str(&body_json)
                    .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

                Ok(HistoryItem {
                    plugin_type,
                    id,
                    body,
                    weight,
                    update_time,
                })
            })?
            .collect::<rusqlite::Result<Vec<HistoryItem<V>>>>()?;

        Ok(result.into_iter().rev().collect())
    }

    pub fn update_or_insert<V: Clone + Serialize>(&self, result: &HistoryItem<V>) -> AResult<()> {
        if let Some(conn) = self.conn.as_ref() {
            let mut stmt = conn.prepare(
                "insert or replace into result_history \
            (id, plugin_type, body_json, weight, update_time) values (?, ?, ?, ?, ?)",
            )?;

            let body_json = serde_json::to_string(&result.body)?;

            stmt.insert(params![
                &result.id,
                &result.plugin_type,
                &body_json,
                &result.weight,
                &result.update_time
            ])?;
        }

        Ok(())
    }

    pub fn try_create_table(&self) -> AResult<()> {
        self.conn
            .context("conn is none")?
            .prepare(
                "CREATE TABLE IF NOT EXISTS result_history (
id TEXT,
plugin_type TEXT,
body_json TEXT,
update_time TIMESTAMP,
weight REAL,
PRIMARY KEY (id)
)",
            )?
            .execute([])?;

        Ok(())
    }
}
