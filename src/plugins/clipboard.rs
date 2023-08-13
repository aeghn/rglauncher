use std::sync::Mutex;
use fragile::Fragile;
use sourceview5;

use gio::{bus_get, Icon};
use glib::Cast;

use gtk::prelude::{DisplayExt, TextBufferExt};
use gtk::{Align, Image, Widget};

use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;
use gtk::traits::GridExt;
use lazy_static::lazy_static;
use rusqlite::Connection;
use gtk::Grid;
use gtk::Label;
use sourceview5::Buffer;

lazy_static!{
    static ref PREVIEW: Mutex<Option<Fragile<(Grid, Label, Label, Label, Buffer)>>> = Mutex::new(None);
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
                                              where content0 like '%{}%' order by UPDATE_TIME desc limit 300", user_input.input.as_str()).as_str());
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
                    .pixel_size(256)
                    .icon_name("xclipboard")
                    .build();
                preview.attach(&image, 0, 0, 1, 3);

                let insert_time = gtk::Label::builder()
                    .halign(Align::End)
                    .build();
                preview.attach(&insert_time, 1, 0, 1, 1);

                let update_time = gtk::Label::builder()
                    .halign(Align::End)
                    .build();
                preview.attach(&update_time, 1, 1, 1, 1);

                let count = gtk::Label::builder()
                    .halign(Align::End)
                    .build();
                preview.attach(&count, 1, 2, 1, 1);

                let buffer = sourceview5::Buffer::builder()
                    .build();

                let label = sourceview5::View::builder()
                    .monospace(true)
                    .show_line_numbers(true)
                    .wrap_mode(gtk::WrapMode::WordChar)
                    .cursor_visible(false)
                    .buffer(&buffer)
                    .hexpand(true)
                    .vexpand(true)
                    .build();
                preview.attach(&label, 0, 3, 2, 1);
                Fragile::new((preview, insert_time, update_time, count, buffer))
            })
            .get();
        let (preview, insert, update, count, buffer) = wv;
        insert.set_label(self.insert_time.as_str());
        update.set_label(self.update_time.as_str());
        count.set_text(self.content.as_str());
        buffer.set_text(self.content.as_str());

        preview.clone().upcast()
    }

    fn on_enter(&self) {
        let display = gtk::gdk::Display::default().unwrap();
        let clipboard = display.clipboard();
        clipboard.set_text(self.content.as_str());
    }
}
