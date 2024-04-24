use std::process::exit;

use rglcore::config::Config;
use rglcore::config::DbConfig;
use rusqlite::Connection;

#[cfg(feature = "wayland")]
use wayland_clipboard_listener::{
    ClipBoardListenContext, ClipBoardListenMessage, WlClipboardPasteStream, WlListenType,
};

#[cfg(feature = "x11")]
use x11_clipboard::Clipboard;

const TEXT: &str = "text/plain;charset=utf-8";
// const IMAGE: &str = "image/png";

pub struct ClipboardWatcher {}

impl ClipboardWatcher {
    pub fn watch(config: Option<DbConfig>) {
        if let Some(db_conf) = config {
            if let Ok(conn) = Connection::open(db_conf.clip_db_path) {
                #[cfg(feature = "wayland")]
                {
                    let mut stream =
                        WlClipboardPasteStream::init(WlListenType::ListenOnCopy).unwrap();
                    for ctx in stream.paste_stream().flatten().flatten() {
                        eprintln!("{ctx:?}");
                        let _ = ClipboardWatcher::try_create_table(&conn);
                        let _ = ClipboardWatcher::winsert(ctx, &conn);
                    }
                }

                #[cfg(feature = "x11")]
                {
                    let clipboard = Clipboard::new().unwrap();

                    // TODO: support image
                    loop {
                        let val = clipboard
                            .load_wait(
                                clipboard.setter.atoms.clipboard,
                                clipboard.setter.atoms.utf8_string,
                                clipboard.setter.atoms.property,
                            )
                            .unwrap();

                        let _ = ClipboardWatcher::try_create_table(&conn);
                        let _ = ClipboardWatcher::xinsert(val, &conn);
                    }
                }
            } else {
                eprintln!("Unable to connect to the database");
                exit(1);
            }
        };
    }

    pub fn id_exist(conn: &Connection, id: &String) -> anyhow::Result<bool> {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM clipboard WHERE id = ?")?;
        let count: i64 = stmt.query_row([&id], |row| row.get(0))?;

        Ok(count > 0)
    }

    #[cfg(feature = "wayland")]
    pub fn winsert(clip: ClipBoardListenMessage, conn: &Connection) -> anyhow::Result<()> {
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

    #[cfg(feature = "x11")]
    pub fn xinsert(clip: Vec<u8>, conn: &Connection) -> anyhow::Result<()> {
        // TODO: SUPPORT IMAGES
        let text = String::from_utf8_lossy(&clip).to_string();
        if !text.is_empty() {
            let id = format!("{:x}", md5::compute(text.clone()));
            if let Ok(exist) = ClipboardWatcher::id_exist(conn, &id) {
                if exist {
                    ClipboardWatcher::update_clipboard(conn, &id, &text)?;
                } else {
                    ClipboardWatcher::insert_clipboard(conn, &id, &text, TEXT)?;
                }
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

fn main() {
    let mut args = std::env::args();
    let program = args.next().expect("Program is always provided by the os");

    match args.next() {
        Some(config_file) => {
            let config = Config::read_from_toml_file(Some(&config_file));
            ClipboardWatcher::watch(config.db);
        }
        None => {
            eprintln!("Usage: {program} <config_file>");
            exit(1);
        }
    }
}
