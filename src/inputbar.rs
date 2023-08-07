use async_broadcast::{Receiver, Sender};
use futures::executor::block_on;
use std::sync::Arc;

use glib::StrV;
use gtk::prelude::EntryExt;
use gtk::traits::EditableExt;
use gtk::{
    self,
    traits::{StyleContextExt, WidgetExt},
};

#[derive(Clone, Debug)]
pub enum InputMessage {
    TextChanged(String),
    EmitSubmit(String),
}

pub struct InputBar {
    pub entry: gtk::Entry,
    pub input_broadcast: Receiver<Arc<InputMessage>>,
    input_sender: Sender<Arc<InputMessage>>,
}

impl InputBar {
    pub fn new() -> Self {
        let (mut input_sender, input_broadcast) = async_broadcast::broadcast(1);
        input_sender.set_overflow(true);

        let entry = gtk::Entry::builder()
            .placeholder_text("Input Anything...")
            .css_classes(StrV::from(vec!["inputbar"]))
            .xalign(0.5)
            .build();

        let tc_tx = input_sender.clone();
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            block_on(async {
                tc_tx
                    .broadcast(Arc::new(InputMessage::TextChanged(text)))
                    .await
            })
            .expect("TODO: panic message");
        });

        let tc_tx = input_sender.clone();
        entry.connect_activate(move |e| {
            let text = e.text().to_string();
            block_on(async {
                tc_tx
                    .broadcast(Arc::new(InputMessage::EmitSubmit(text)))
                    .await
            })
            .expect("TODO: panic message");
        });

        InputBar {
            entry,
            input_broadcast,
            input_sender,
        }
    }
}
