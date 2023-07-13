use glib::{Sender};
use gtk::{self, traits::{WidgetExt, StyleContextExt}};
use gtk::traits::EditableExt;
use crate::shared::UserInput;


pub fn get_input_bar(tx: &Sender<UserInput>) -> gtk::Entry {
    let input_bar = gtk::Entry::builder()
        .placeholder_text("input anything")
        .build();
    
    input_bar.style_context().add_class("inputbar");

    let tx = tx.clone();
    input_bar.connect_changed(move |e| {
        let text = e.text().to_string();
        tx.send(UserInput::new(text.as_str())).expect("unable to get text from input bar");
    });

    input_bar
}
