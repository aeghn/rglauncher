use futures::executor::block_on;
use std::sync::Arc;

use glib::StrV;
use gtk;
use gtk::prelude::EntryExt;
use gtk::traits::EditableExt;

#[derive(Clone, Debug)]
pub enum InputMessage {
    TextChanged(String),
    EmitSubmit(String),
    RefreshContent,
}

#[derive(Clone)]
pub struct InputBar {
    pub entry: gtk::Entry,
}

impl InputBar {
    pub fn new(input_sender: &flume::Sender<Arc<InputMessage>>) -> Self {
        let entry = gtk::Entry::builder()
            .placeholder_text("Input Anything...")
            .css_classes(StrV::from(vec!["inputbar"]))
            .xalign(0.5)
            .build();

        let tx = input_sender.clone();
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            block_on(async {
                tx.send(Arc::new(InputMessage::TextChanged(text)))
            })
            .expect("TODO: panic message");
        });

        let tx = input_sender.clone();
        entry.connect_activate(move |e| {
            let text = e.text().to_string();
            block_on(async { tx.send(Arc::new(InputMessage::EmitSubmit(text))) })
                .expect("TODO: panic message");
        });

        InputBar { entry }
    }
}
