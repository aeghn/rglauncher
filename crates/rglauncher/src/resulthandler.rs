use crate::preview::PreviewMsg;
use crate::sidebar::{Sidebar, SidebarMsg};
use backend::plugindispatcher::{DispatchMsg, PluginDispatcher};
use backend::plugins::PluginResult;
use backend::userinput::UserInput;
use backend::ResultMsg;
use flume::Sender;
use futures::executor::block_on;
use futures::sink::drain;
use glib::BoxedAnyObject;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use tracing::{debug, error, info};

pub struct ResultHolder {
    user_input: Option<Arc<UserInput>>,
    current_index: Option<u32>,
    result_holder: Vec<Arc<dyn PluginResult>>,

    pub result_sender: flume::Sender<ResultMsg>,
    result_receiver: flume::Receiver<ResultMsg>,

    dispatch_sender: flume::Sender<DispatchMsg>,

    sidebar_sender: flume::Sender<SidebarMsg>,
    preview_sender: Sender<PreviewMsg>,
}

impl ResultHolder {
    fn new(
        dispatch_sender: flume::Sender<DispatchMsg>,
        sidebar_sender: Sender<SidebarMsg>,
        preview_sender: Sender<PreviewMsg>,
    ) -> Self {
        let (result_sender, result_receiver) = flume::unbounded();

        Self {
            user_input: None,
            current_index: None,
            result_holder: vec![],

            result_sender,
            result_receiver,
            dispatch_sender,
            sidebar_sender,
            preview_sender,
        }
    }

    fn send_to_sidebar(&mut self) {
        self.result_holder
            .sort_by(|e1, e2| e2.score().cmp(&e1.score()));
        let holder = self.result_holder.clone();
        self.sidebar_sender
            .send(SidebarMsg::Result(holder))
            .expect("unable to send result to sidebar")
    }

    async fn accept_messages(&mut self) {
        loop {
            match self.result_receiver.recv_async().await {
                Ok(msg) => match msg {
                    ResultMsg::Result(input, mut results) => match self.user_input.as_ref() {
                        None => {}
                        Some(user_input) => {
                            if user_input.as_ref() == input.as_ref() {
                                self.result_holder.append(&mut results);
                                self.send_to_sidebar();
                            }
                        }
                    },
                    ResultMsg::UserInput(input) => {
                        if let Some(mut old_input) = self.user_input.replace(input.clone()) {
                            old_input.cancel();
                            self.current_index.take();
                            self.result_holder.clear();
                        }
                        debug!("Send message to dispatcher: {}", input.input);
                        self.dispatch_sender
                            .send(DispatchMsg::UserInput(
                                input.clone(),
                                self.result_sender.clone(),
                            ))
                            .expect("todo");
                    }
                    ResultMsg::RemoveWindow => {}
                    ResultMsg::ChangeSelect(item) => {
                        self.current_index.replace(item.clone());
                        match self.result_holder.get(item as usize) {
                            Some(pr) => {
                                self.preview_sender
                                    .send(PreviewMsg::PluginResult(pr.clone()))
                                    .expect("unable to send preview msg");
                            }
                            _ => {}
                        }
                    }
                },
                Err(ex) => {
                    error!("unable to receive message: {:?}", ex);
                }
            }
        }
    }

    pub fn start(
        dispatcher_sender: &flume::Sender<DispatchMsg>,
        sidebar_sender: &flume::Sender<SidebarMsg>,
        preview_sender: &Sender<PreviewMsg>,
    ) -> Sender<ResultMsg> {
        let mut result_handler = Self::new(
            dispatcher_sender.clone(),
            sidebar_sender.clone(),
            preview_sender.clone(),
        );

        let result_sender = result_handler.result_sender.clone();

        thread::spawn(move || {
            block_on(result_handler.accept_messages());
        });

        result_sender
    }
}
