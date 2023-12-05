use anyhow::Error;
use chrono::{DateTime, Local, Utc};
use fragile::Fragile;
use std::sync::Mutex;

use glib::Cast;

use gtk::prelude::{DisplayExt, TextBufferExt};
use gtk::Align::End;
use gtk::{Align, Image, TextBuffer, TextView, Widget};

use crate::plugins::{Plugin, PluginPreview, PluginResult};
use crate::userinput::UserInput;
use crate::util::score_utils;
use gtk::traits::{GridExt, WidgetExt};
use gtk::WrapMode::WordChar;
use lazy_static::lazy_static;
use rusqlite::Connection;

#[derive(Debug)]
pub struct ClipResult {
    content: String,
    score: i32,
    mime: String,
    insert_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
    count: i64,
}

impl PluginResult for ClipResult {
    fn score(&self) -> i32 {
        score_utils::middle(self.score as i64)
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
        if let Some(display) = gtk::gdk::Display::default() {
            let clipboard = display.clipboard();
            clipboard.set_text(self.content.as_str());
        }
    }
}

pub struct ClipboardPlugin {
    connection: Option<Connection>,
}

impl ClipboardPlugin {
    pub fn new(path: &str) -> Self {
        if !std::path::Path::new(path).exists() {
            return ClipboardPlugin { connection: None };
        }

        let conn = match Connection::open(path) {
            Ok(e) => Some(e),
            Err(_) => None,
        };

        return ClipboardPlugin { connection: conn };
    }
}

impl Plugin<ClipResult> for ClipboardPlugin {
    fn refresh_content(&mut self) {}

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<ClipResult>> {
        if let Some(conn) = self.connection.as_ref() {
            let mut stmt = conn.prepare(
                "SELECT content0, mimes, insert_time, update_time, count \
        from clipboard where content0 like ? order by UPDATE_TIME asc limit 100",
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
                    })
                })?
                .collect::<Result<Vec<ClipResult>, rusqlite::Error>>()?;

            Ok(result)
        } else {
            Err(Error::msg("unable to find connection"))
        }
    }
}

pub struct ClipPreview {
    root: gtk::Grid,
    insert_time: gtk::Label,
    update_time: gtk::Label,
    count: gtk::Label,
    mime: gtk::Label,
    text_buffer: gtk::TextBuffer,
}

impl PluginPreview<ClipResult> for ClipPreview {
    fn new() -> Self {
        let preview = gtk::Grid::builder().vexpand(true).hexpand(true).build();

        let image = Image::builder()
            .pixel_size(48)
            .icon_name("xclipboard")
            .halign(Align::End)
            .build();
        preview.attach(&image, 0, 0, 1, 4);

        let f = |e| gtk::Label::builder().label(e).halign(End).build();

        preview.attach(&f("Insert Time: "), 1, 0, 1, 1);
        let insert_time = gtk::Label::builder().halign(Align::Start).build();
        preview.attach(&insert_time, 2, 0, 1, 1);

        preview.attach(&f("Update Time: "), 1, 1, 1, 1);
        let update_time = gtk::Label::builder().halign(Align::Start).build();
        preview.attach(&update_time, 2, 1, 1, 1);

        preview.attach(&f("Insert Count: "), 1, 2, 1, 1);
        let count = gtk::Label::builder().halign(Align::Start).build();
        preview.attach(&count, 2, 2, 1, 1);

        preview.attach(&f("Mime: "), 1, 3, 1, 1);
        let mime = gtk::Label::builder().halign(Align::Start).build();
        preview.attach(&mime, 2, 3, 1, 1);

        let text_buffer = TextBuffer::builder().build();
        let text_view = TextView::builder()
            .hexpand(true)
            .vexpand(true)
            .wrap_mode(WordChar)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .buffer(&text_buffer)
            .build();
        preview.attach(&text_view, 0, 4, 3, 1);

        ClipPreview {
            root: preview,
            insert_time,
            update_time,
            count,
            mime,
            text_buffer,
        }
    }

    fn get_preview(&self, plugin_result: ClipResult) -> Widget {
        let il: DateTime<Local> = DateTime::from(plugin_result.insert_time);
        self.insert_time.set_label(il.to_string().as_str());
        let il: DateTime<Local> = DateTime::from(plugin_result.update_time);
        self.update_time.set_label(il.to_string().as_str());
        self.count
            .set_text(plugin_result.count.to_string().as_str());
        self.text_buffer.set_text(plugin_result.content.as_str());
        self.mime.set_text(plugin_result.mime.as_str());

        self.root.clone().upcast()
    }
}

crate::register_plugin_preview!(ClipResult, ClipPreview);
