use crate::pluginpreview::PluginPreview;
use backend::plugins::clipboard::ClipResult;
use chrono::{DateTime, Local};
use glib::Cast;
use gtk::prelude::{GridExt, TextBufferExt};
use gtk::Align::End;
use gtk::WrapMode::WordChar;
use gtk::{Align, Image, TextBuffer, TextView, Widget};

pub struct ClipPreview {
    root: gtk::Grid,
    insert_time: gtk::Label,
    update_time: gtk::Label,
    count: gtk::Label,
    mime: gtk::Label,
    text_buffer: gtk::TextBuffer,
}

impl PluginPreview for ClipPreview {
    type PluginResult = ClipResult;
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

    fn get_preview(&self, plugin_result: &ClipResult) -> Widget {
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
