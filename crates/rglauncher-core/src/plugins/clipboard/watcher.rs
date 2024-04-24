use tracing::error;

use crate::config::DbConfig;
use rusqlite::Connection;
use wayland_clipboard_listener::WlClipboardPasteStream;
use wayland_clipboard_listener::{ClipBoardListenContext, ClipBoardListenMessage, WlListenType};

const TEXT: &str = "text/plain;charset=utf-8";
// const IMAGE: &str = "image/png";

pub struct ClipboardWatcher {}

impl ClipboardWatcher {
    pub fn watch(config: Option<DbConfig>) {
        if let Some(db_conf) = config {
            if let Ok(conn) = Connection::open(db_conf.db_path) {
                let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy).unwrap();
                for ctx in stream.paste_stream().flatten().flatten() {
                    let _ = ClipboardWatcher::try_create_table(&conn);
                    let _ = ClipboardWatcher::insert(ctx, &conn);
                }
            } else {
                error!("Unable to connect to the database");
            }
        };
    }

    pub fn id_exist(conn: &Connection, id: &String) -> anyhow::Result<bool> {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM clipboard WHERE id = ?")?;
        let count: i64 = stmt.query_row([&id], |row| row.get(0))?;

        Ok(count > 0)
    }

    pub fn insert(clip: ClipBoardListenMessage, conn: &Connection) -> anyhow::Result<()> {
        // TODO: SUPPORT IMAGES
        if clip.mime_types.contains(&TEXT.to_string()) {
            match clip.context {
                ClipBoardListenContext::Text(text) => {
                    let id = format!("{:x}", md5::compute(text.clone()));
                    if let Ok(exist) = ClipboardWatcher::id_exist(conn, &id) {
                        if exist {
                            ClipboardWatcher::update_clipboard(conn, &id, &text)?;
                        } else {
                            ClipboardWatcher::insert_clipboard(conn, &id, &text, TEXT)?;
                        }
                    }
                }
                ClipBoardListenContext::File(_) => {}
            }
        }

        Ok(())
    }

    pub fn update_clipboard(conn: &Connection, id: &str, content: &str) -> anyhow::Result<()> {
        conn.execute(
            "UPDATE clipboard
         SET count = count + 1,
             update_time = CURRENT_TIMESTAMP
         WHERE id = ? AND content = ?",
            [id, content],
        )?;
        Ok(())
    }

    pub fn insert_clipboard(
        conn: &Connection,
        id: &str,
        content: &str,
        mime: &str,
    ) -> anyhow::Result<()> {
        conn.execute(
            "INSERT INTO clipboard (id, content, mime, insert_time, update_time, count)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 1)",
            [id, content, mime],
        )?;
        Ok(())
    }

    pub fn try_create_table(conn: &Connection) -> anyhow::Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard (
            id TEXT,
            content TEXT,
            mime TEXT,
            insert_time TIMESTAMP,
            update_time TIMESTAMP,
            count INTEGER,
            PRIMARY KEY (id, content)
        )",
            [],
        )?;

        Ok(())
    }
}
