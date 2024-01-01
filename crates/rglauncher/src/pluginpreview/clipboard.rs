use crate::pluginpreview::PluginPreview;
use backend::plugins::clipboard::ClipResult;
use chrono::{DateTime, Local};
use glib::Cast;
use gtk::prelude::{BoxExt, GridExt, TextBufferExt};
use gtk::Align::End;
use gtk::WrapMode::WordChar;
use gtk::{Align, Image, Orientation, Overflow, TextBuffer, TextView, Widget};

pub struct ClipPreview {
    root: gtk::Box,
    insert_time: gtk::Label,
    update_time: gtk::Label,
    count: gtk::Label,
    mime: gtk::Label,
    text_buffer: gtk::TextBuffer,
}

impl PluginPreview for ClipPreview {
    type PluginResult = ClipResult;
    fn new() -> Self {
        let preview = gtk::Box::builder()
            .hexpand(true)
            .vexpand(true)
            .orientation(Orientation::Vertical)
            .build();

        let text_buffer = TextBuffer::builder().build();
        let text_view = TextView::builder()
            .hexpand(true)
            .wrap_mode(WordChar)
            .css_classes(["raw-box"])
            .buffer(&text_buffer)
            .vexpand(false)
            .focusable(false)
            .build();

        let text_window = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        text_window.set_child(Some(&text_view));

        let sep = super::get_seprator();

        let info_grid = gtk::Grid::builder()
            .hexpand(true)
            .vexpand(false)
            .valign(End)
            .build();

        let insert_time = super::build_pair_line(&info_grid, 0, "Insert Time: ");

        let update_time = super::build_pair_line(&info_grid, 1, "Update Time: ");

        let count = super::build_pair_line(&info_grid, 2, "Insert Count: ");

        let mime = super::build_pair_line(&info_grid, 3, "Mime: ");

        preview.append(&text_window);
        preview.append(&sep);
        preview.append(&info_grid);

        ClipPreview {
            root: preview,
            insert_time,
            update_time,
            count,
            mime,
            text_buffer,
        }
    }

    fn get_preview(&self) -> Widget {
        self.root.clone().upcast()
    }

    fn set_preview(&self, plugin_result: &Self::PluginResult) {
        let il: DateTime<Local> = DateTime::from(plugin_result.insert_time);
        self.insert_time.set_label(il.to_string().as_str());
        let il: DateTime<Local> = DateTime::from(plugin_result.update_time);
        self.update_time.set_label(il.to_string().as_str());
        self.count
            .set_text(plugin_result.count.to_string().as_str());
        self.text_buffer.set_text(plugin_result.content.as_str());
        self.mime.set_text(plugin_result.mime.as_str());
    }

    fn get_id(&self) -> &str {
        backend::plugins::clipboard::TYPE_ID
    }
}
