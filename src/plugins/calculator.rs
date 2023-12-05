use fragile::Fragile;
use gtk::{TextBuffer, Widget};
use gtk::prelude::{GridExt, TextBufferExt};
use gtk::WrapMode::WordChar;
use lazy_static::lazy_static;

use crate::plugins::{Plugin, PluginPreview, PluginResult};
use crate::userinput::UserInput;
use glib::{Cast};
use gtk::prelude::{WidgetExt};


use std::option::Option::None;

use std::sync::Mutex;
use crate::util::score_utils;

pub struct CalcResult {
    pub formula: String,
    pub result: String
}

impl PluginResult for CalcResult {
    fn score(&self) -> i32 {
        score_utils::highest()
    }

    fn sidebar_icon_name(&self) -> String {
        "calc".to_string()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some("calc".to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(self.formula.to_string())
    }

    fn on_enter(&self) {

    }
}

pub struct Calculator {

}

impl Calculator {
    pub fn new() -> Self {
        Calculator {}
    }
}

impl Plugin<CalcResult> for Calculator {
    fn refresh_content(&mut self) {

    }

    fn handle_input(&self, user_input: &UserInput) -> anyhow::Result<Vec<CalcResult>> {
        Ok(vec![meval::eval_str(user_input.input.as_str())
            .map(|res| {
                CalcResult {
                    formula: user_input.input.clone(),
                    result: res.to_string()
                }
            })?])
    }
}

pub struct CalcPreview {
    root: gtk::Grid,
    formula_buffer: gtk::TextBuffer,
    result_buffer: gtk::TextBuffer
}

impl PluginPreview<CalcResult> for CalcPreview {
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
            result_buffer
        }
    }

    fn get_preview(&self, plugin_result: CalcResult) -> gtk::Widget {
        self.formula_buffer.set_text(plugin_result.formula.as_str());
        self.result_buffer.set_text(plugin_result.result.as_str());

        self.root.clone().upcast()
    }
}

crate::register_plugin_preview!(CalcResult, CalcPreview);