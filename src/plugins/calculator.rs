use fragile::Fragile;
use gtk::{TextBuffer, Widget};
use gtk::prelude::{GridExt, TextBufferExt};
use gtk::WrapMode::WordChar;
use lazy_static::lazy_static;
use meval::Error;
use crate::plugins::{Plugin, PluginResult};
use crate::user_input::UserInput;
use glib::{Cast, StrV};
use gtk::prelude::{WidgetExt};
use gtk::Align::Center;
use gtk::Grid;
use gtk::{Image, Label};
use std::option::Option::None;

use std::sync::Mutex;
use tracing::error;
lazy_static! {
    static ref PREVIEW: Mutex<Option<Fragile<(Grid, TextBuffer, TextBuffer)>>> = Mutex::new(None);
}

pub struct Calculator {

}

impl Calculator {
    pub fn new() -> Self {
        Calculator {}
    }
}

pub struct CalcResult {
    pub formula: String,
    pub result: String
}

impl PluginResult for CalcResult {
    fn get_score(&self) -> i32 {
        1000
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

    fn preview(&self) -> Widget {
        let mut guard = PREVIEW.lock().unwrap();
        let wv = guard.get_or_insert_with(|| {
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

            Fragile::new((preview, formula_buffer, result_buffer))
        }).get();

        let (preview, formula, result) = wv;
        formula.set_text(self.formula.as_str());
        result.set_text(self.result.as_str());

        preview.clone().upcast()
    }

    fn on_enter(&self) {

    }
}

impl Plugin<CalcResult> for Calculator {
    fn handle_input(&self, user_input: &UserInput) -> Vec<CalcResult> {
        let mut vec = vec![];
        match meval::eval_str(user_input.input.as_str()) {
            Ok(res) => {
                vec.push(CalcResult{
                    formula: user_input.input.to_string(),
                    result: res.to_string(),
                });
            }
            Err(_) => {}
        }

        vec
    }
}