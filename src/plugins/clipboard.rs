use fragile::Fragile;
use std::sync::Mutex;

use glib::Cast;

use gtk::prelude::{DisplayExt, TextBufferExt};
use gtk::{Align, Image, TextBuffer, TextView, Widget};

use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;
use gtk::traits::GridExt;
use gtk::Grid;
use gtk::Label;
use gtk::WrapMode::WordChar;
use lazy_static::lazy_static;
use rusqlite::Connection;

lazy_static! {
    static ref PREVIEW: Mutex<Option<Fragile<(Grid, Label, Label, Label, Label, TextBuffer)>>> =
        Mutex::new(None);
}

pub struct ClipboardPlugin {
    conn: Option<Connection>,
}

#[derive(Debug)]
pub struct ClipPluginResult {
    content: String,
    score: i32,
    mime: String,
    insert_time: String,
    update_time: String,
    count: i64,
}

impl ClipboardPlugin {
    pub fn new(path: &str) -> Self {
        if !std::path::Path::new(path).exists() {
            return ClipboardPlugin { conn: None };
        }

        let conn = match Connection::open(path) {
            Ok(e) => Some(e),
            Err(_) => None,
        };

        return ClipboardPlugin { conn };
    }
}

impl Plugin<ClipPluginResult> for ClipboardPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<ClipPluginResult> {
        let mut vec: Vec<ClipPluginResult> = vec![];

        if let Some(_conn) = &self.conn {
            let stmt = _conn.prepare(format!("SELECT content0, mimes, insert_time, update_time, count from clipboard \
                                              where content0 like '%{}%' order by UPDATE_TIME asc limit 100", user_input.input.as_str()).as_str());
            if let Ok(mut _stmt) = stmt {
                let iter = _stmt.query_map([], |row| {
                    Ok(ClipPluginResult {
                        content: row.get(0).unwrap(),
                        score: 0,
                        mime: row.get(1).unwrap(),
                        insert_time: row.get(2).unwrap(),
                        update_time: row.get(3).unwrap(),
                        count: row.get(4).unwrap(),
                    })
                });
                if let Ok(_iter) = iter {
                    for cpr in _iter {
                        vec.push(cpr.unwrap());
                    }
                }
            }
        };
        vec
    }
}

impl PluginResult for ClipPluginResult {
    fn get_score(&self) -> i32 {
        self.score
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

    fn preview(&self) -> Widget {
        let mut guard = PREVIEW.lock().unwrap();

        let wv = guard
            .get_or_insert_with(|| {
                let preview = gtk::Grid::builder().vexpand(true).hexpand(true).build();

                let image = Image::builder()
                    .pixel_size(48)
                    .icon_name("xclipboard")
                    .halign(Align::Center)
                    .build();
                preview.attach(&image, 0, 0, 1, 3);

                preview.attach(&gtk::Label::new(Some("Insert Time")), 1, 0, 1, 1);
                let insert_time = gtk::Label::builder().halign(Align::Start).build();
                preview.attach(&insert_time, 2, 0, 1, 1);

                preview.attach(&gtk::Label::new(Some("Update Time")), 1, 1, 1, 1);
                let update_time = gtk::Label::builder().halign(Align::Start).build();
                preview.attach(&update_time, 2, 1, 1, 1);

                preview.attach(&gtk::Label::new(Some("Insert Count")), 1, 2, 1, 1);
                let count = gtk::Label::builder().halign(Align::Start).build();
                preview.attach(&count, 2, 2, 1, 1);

                preview.attach(&gtk::Label::new(Some("Mime")), 1, 3, 1, 1);
                let mime = gtk::Label::builder().halign(Align::Start).build();
                preview.attach(&mime, 2, 3, 1, 1);

                let text_buffer = TextBuffer::builder().build();
                let text_view = TextView::builder()
                    .hexpand(true)
                    .vexpand(true)
                    .wrap_mode(WordChar)
                    .buffer(&text_buffer)
                    .build();

                preview.attach(&text_view, 0, 3, 3, 1);

                Fragile::new((preview, insert_time, update_time, count, mime, text_buffer))
            })
            .get();
        let (preview, insert, update, count, mime, buffer) = wv;
        insert.set_label(self.insert_time.as_str());
        update.set_label(self.update_time.as_str());
        count.set_text(self.count.to_string().as_str());
        buffer.set_text(self.content.as_str());
        mime.set_text(self.mime.as_str());

        preview.clone().upcast()
    }

    fn on_enter(&self) {
        let display = gtk::gdk::Display::default().unwrap();
        let clipboard = display.clipboard();
        clipboard.set_text(self.content.as_str());
    }
}
