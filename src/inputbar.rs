use barrage::{Receiver, Sender};
use glib::{GString, StrV};
use gtk::{self, traits::{WidgetExt, StyleContextExt}};
use gtk::prelude::EntryExt;
use gtk::traits::EditableExt;
use tracing::error;
use crate::sidebar::SidebarMsg;

#[derive(Clone, Debug)]
pub enum InputMessage {
    TextChanged(String),
    EmitSubmit(String)
}

pub struct InputBar {
    pub entry: gtk::Entry,
    pub input_boardcast: Receiver<InputMessage>,
    input_sender: Sender<InputMessage>,
}

impl InputBar {
    pub fn new() -> Self {
        let (input_sender, input_boardcast) = barrage::unbounded::<InputMessage>();
        let entry = gtk::Entry::builder()
            .placeholder_text("Input Anything...")
            .css_classes(StrV::from(vec!["inputbar"]))
            .xalign(0.5)
            .build();



        let tc_tx = input_sender.clone();
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            match tc_tx.send(InputMessage::TextChanged(text)) {
                Ok(_) => {}
                Err(err) => {
                }
            }
        });

        let tc_tx = input_sender.clone();
        entry.connect_activate(move |e| {
            tc_tx.send(InputMessage::EmitSubmit(e.text().to_string()));
        });

        InputBar {
            entry,
            input_boardcast,
            input_sender
        }
    }

}

