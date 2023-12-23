use backend::userinput::UserInput;
use backend::ResultMsg;
use futures::executor::block_on;
use std::sync::Arc;

use glib::StrV;
use gtk;
use gtk::prelude::EntryExt;
use gtk::traits::EditableExt;
use tracing::info;

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
    pub fn new(result_sender: &flume::Sender<ResultMsg>, window_id: i32) -> Self {
        let entry = gtk::Entry::builder()
            .placeholder_text("Input Anything...")
            .css_classes(StrV::from(vec!["inputbar"]))
            .xalign(0.5)
            .build();

        let result_sender = result_sender.clone();
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            result_sender
                .send(ResultMsg::UserInput(Arc::new(UserInput::new(
                    &text, &window_id,
                ))))
                .expect("TODO: panic message");
        });

        // entry.connect_activate(move |e| {
        //     let text = e.text().to_string();
        //     block_on(async { input_tx.send(Arc::new(InputMessage::EmitSubmit(text))) })
        //         .expect("TODO: panic message");
        // });

        InputBar { entry }
    }
}
