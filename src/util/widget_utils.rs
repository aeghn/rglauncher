use glib::GString;

use gtk::Label;
use gtk::pango::WrapMode::WordChar;

pub fn get_wrapped_label(text: &str, xalign: f32) -> Label {
    let label_builder = Label::builder();
    label_builder.label(GString::from(text))
        .wrap(true)
        .wrap_mode(WordChar)
        .hexpand(true)
        .xalign(xalign)
        .build()
}