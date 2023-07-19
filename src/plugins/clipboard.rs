

use gio::Icon;
use glib::Cast;

use gtk::{Grid, Label, Widget};
use gtk::pango::WrapMode::WordChar;
use gtk::traits::GridExt;
use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;
use rusqlite::{Connection};
use tracing::error;


pub struct ClipboardPlugin {
    conn: Option<Connection>
}

#[derive(Debug)]
pub struct ClipPluginResult {
    id: u64,
    content: String,
    score: i32,
    mime: String,
    insert_time: String
}

impl ClipboardPlugin {
    pub fn new(path: &str) -> Self {
        if !std::path::Path::new(path).exists() {
            return ClipboardPlugin { conn: None }
        }

        let conn = match Connection::open(path) {
            Ok(e) => {
                Some(e)
            }
            Err(_) => {
                None
            }
        };

        return ClipboardPlugin { conn }
    }
}


impl Plugin for ClipboardPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<Box<dyn PluginResult>> {
        error!("begin to collect: {:?}", user_input);
        let mut vec: Vec<Box<dyn PluginResult>> = vec![];

        if let Some(_conn) = &self.conn {
            let stmt = _conn.prepare(format!("SELECT id, content0, mimes, insert_time from clipboard \
            where content0 like '%{}%' order by INSERT_TIME desc limit 1000", user_input.input.as_str()).as_str());
            if let Ok(mut _stmt) =stmt {
                let iter = _stmt.query_map([], |row| {
                    Ok(ClipPluginResult {
                        id: row.get(0).unwrap(),
                        content: row.get(1).unwrap(),
                        score: 0,
                        mime: row.get(2).unwrap(),
                        insert_time: row.get(3).unwrap()
                    })
                });
                if let Ok(_iter) = iter {
                    for cpr in _iter {
                        vec.push(Box::new(cpr.unwrap()));
                    }
                }
            }
        };
        error!("end to collect: {:?}", user_input);
        vec
    }
}

impl ClipPluginResult {
    pub fn new() -> Self {
        Self{
            id: 0,
            content: "".to_string(),
            score: 0,
            mime: "".to_string(),
            insert_time: "".to_string(),
        }
    }
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}


impl PluginResult for ClipPluginResult {
    fn get_score(&self) -> i32 {
        self.score
    }

    fn sidebar_icon(&self) -> Option<Icon> {
        Some(gio::Icon::from(gio::ThemedIcon::from_names(&[&"xclipboard"])))
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.insert_time.to_string())
    }

    fn sidebar_content(&self) -> Option<Widget> {
        let label = Label::new(Some(truncate(self.content.trim(), 60)));
        label.set_wrap(true);
        label.set_wrap_mode(WordChar);
        Some(label.upcast())
    }

    fn preview(&self) -> Grid {
        let preview = gtk::Grid::new();
        let label = Label::new(Some(self.content.as_str()));
        label.set_wrap(true);
        label.set_wrap_mode(WordChar);
        preview.attach(&label, 0, 0, 1, 1);

        preview
    }

    fn on_enter(&self) {
        todo!()
    }
}