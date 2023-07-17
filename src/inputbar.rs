use glib::{clone, GString};
use flume::Sender;
use gtk::{self, traits::{WidgetExt, StyleContextExt}};
use gtk::prelude::EntryExt;
use gtk::traits::EditableExt;
use crate::shared::UserInput;

pub enum InputMessage {
    TextChange(String),
    EmitEnter
}

pub fn get_input_bar(tx: &Sender<InputMessage>) -> gtk::Entry {
    let input_bar = gtk::Entry::builder()
        .placeholder_text("Input anything")
        .css_name(GString::from("inputbar"))
        .xalign(0.5)
        .build();

    input_bar.connect_changed(move |e| {
        let text = e.text().to_string();
        tx.send(InputMessage::TextChange(text))
            .expect("unable to get text from input bar");
    });

    input_bar.connect_activate(|| {
        tx.send(InputMessage::EmitEnter)
            .expect("unable to send enter signal");
    });

    input_bar
}
