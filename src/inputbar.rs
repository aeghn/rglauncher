
use glib::{GString};
use flume::{Receiver, Sender};
use gtk::{self, traits::{WidgetExt, StyleContextExt}};
use gtk::prelude::EntryExt;
use gtk::traits::EditableExt;
use tracing::error;


pub enum InputMessage {
    TextChange(String),
    EmitEnter
}

pub struct InputBar {
    pub entry: gtk::Entry,
    pub input_receiver: Receiver<InputMessage>,
    input_sender: Sender<InputMessage>,
}

impl InputBar {
    pub fn new() -> Self {
        let entry = gtk::Entry::builder()
            .placeholder_text("Input Anything...")
            .css_classes(&["inputbar"])
            .xalign(0.5)
            .build();

        let (input_sender, input_receiver) = flume::unbounded::<InputMessage>();

        let tc_tx = input_sender.clone();
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            match tc_tx.send(InputMessage::TextChange(text)) {
                Ok(_) => {}
                Err(err) => {
                    error!("err is {:?}", err);
                }
            }
        });

        entry.connect_activate(move |_| {
            input_sender.send(InputMessage::EmitEnter)
                .expect("unable to send enter signal");
        });

        InputBar {
            entry,
            input_receiver,
            input_sender
        }
    }

}

