use glib::GString;

use crate::util::string_utils;
use gtk::pango::WrapMode::WordChar;
use gtk::Label;

pub fn get_wrapped_label(text: &str, xalign: f32) -> Label {
    let label_builder = Label::builder();
    label_builder
        .label(GString::from(text))
        .wrap(true)
        .wrap_mode(WordChar)
        .hexpand(true)
        .xalign(xalign)
        .build()
}

pub fn limit_length_label(text: &str, limit: usize, xalign: f32) -> Label {
    let limited_text = string_utils::truncate(text, limit);
    get_wrapped_label(limited_text, xalign)
}
