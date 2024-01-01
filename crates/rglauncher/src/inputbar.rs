use backend::userinput::UserInput;
use backend::ResultMsg;
use futures::executor::block_on;
use std::sync::Arc;
use flume::{RecvError, Sender};

use glib::{ControlFlow, MainContext, StrV};
use gtk;
use gtk::Align::Center;
use gtk::prelude::{EntryExt, WidgetExt};
use gtk::traits::EditableExt;
use tracing::info;
use crate::window::WindowMsg;

#[derive(Clone, Debug)]
pub enum InputMessage {
    TextChanged(String),
    TextAppend(String),
    Clear,
    Focus
}

#[derive(Clone)]
pub struct InputBar {
    pub entry: gtk::Entry,
    pub input_sender: flume::Sender<InputMessage>,
    input_receiver: flume::Receiver<InputMessage>
}

impl InputBar {
    pub fn new(result_sender: &flume::Sender<ResultMsg>,
               window_sender: &flume::Sender<WindowMsg>,
               window_id: i32) -> Self {
        let (input_sender, input_receiver) = flume::unbounded();

        let entry = gtk::Entry::builder()
            .placeholder_text("Input Anything...")
            .css_classes(StrV::from(vec!["inputbar"]))
            .halign(Center)
            .build();

        {
            let result_sender = result_sender.clone();
            entry.connect_changed(move |e| {
                let text = e.text().to_string();
                result_sender
                    .send(ResultMsg::UserInput(Arc::new(UserInput::new(
                        &text, &window_id,
                    ))))
                    .expect("TODO: panic message");
            });
        }

        {
            let result_sender = result_sender.clone();
            let window_tx = window_sender.clone();
            entry.connect_activate(move |e| {
                result_sender.send(ResultMsg::SelectSomething).expect("TODO: panic message");
                window_tx.send(WindowMsg::Close).expect("unable to close window");
            });
        }

        {
            let input_rx = input_receiver.clone();
            let entry = entry.clone();
            MainContext::ref_thread_default().spawn_local( async move {
                match input_rx.recv_async().await {
                    Ok(input_msg) => {
                        match input_msg {
                            InputMessage::TextChanged(input) => {
                                entry.set_text(input.as_str());
                            }
                            InputMessage::Clear => {
                                entry.set_text("");
                            }
                            InputMessage::TextAppend(cs) => {
                                let mut pos: i32 = entry.text_length() as i32;
                                entry.insert_text(cs.as_str(), &mut pos);
                            }
                            InputMessage::Focus => {
                                entry.grab_focus_without_selecting();
                            }
                        }
                    }
                    Err(_) => {}
                }

                ControlFlow::Continue
            });
        }

        InputBar { entry, input_sender, input_receiver }
    }

}
