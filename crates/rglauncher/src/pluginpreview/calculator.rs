use crate::pluginpreview::PluginPreview;
use backend::plugins::calculator::CalcResult;
use glib::Cast;
use gtk::prelude::{GridExt, TextBufferExt};
use gtk::WrapMode::WordChar;

pub struct CalcPreview {
    root: gtk::Grid,
    formula_buffer: gtk::TextBuffer,
    result_buffer: gtk::TextBuffer,
}

impl PluginPreview for CalcPreview {
    type PluginResult = CalcResult;
    fn new() -> Self {
        let preview = gtk::Grid::builder().vexpand(true).hexpand(true).build();

        let formula_buffer = gtk::TextBuffer::builder().build();
        let formula_area = gtk::TextView::builder()
            .hexpand(true)
            .vexpand(true)
            .wrap_mode(WordChar)
            .margin_start(15)
            .margin_end(15)
            .margin_top(15)
            .margin_bottom(15)
            .buffer(&formula_buffer)
            .build();
        preview.attach(&formula_area, 0, 0, 1, 1);

        let result_buffer = gtk::TextBuffer::builder().build();
        let result_area = gtk::TextView::builder()
            .hexpand(true)
            .vexpand(true)
            .wrap_mode(WordChar)
            .margin_start(15)
            .margin_end(15)
            .margin_top(15)
            .margin_bottom(15)
            .buffer(&result_buffer)
            .build();
        preview.attach(&result_area, 0, 1, 1, 1);

        CalcPreview {
            root: preview,
            formula_buffer,
            result_buffer,
        }
    }

    fn get_preview(&self, plugin_result: &CalcResult) -> gtk::Widget {
        self.formula_buffer.set_text(plugin_result.formula.as_str());
        self.result_buffer.set_text(plugin_result.result.as_str());

        self.root.clone().upcast()
    }
}
