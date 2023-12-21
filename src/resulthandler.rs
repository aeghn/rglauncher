use futures::executor::block_on;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tracing::error;

use crate::{plugins::PluginResult, userinput::UserInput};

pub enum ResultMsg {
    Result(Arc<UserInput>, Vec<Box<dyn PluginResult>>),
    UserInput(Arc<UserInput>),
    RemoveWindow(i32),
}

struct ResultHolder {
    user_input: Arc<UserInput>,
    result_holder: Vec<Box<dyn PluginResult>>,
}

impl ResultHolder {
    pub fn new(user_input: Arc<UserInput>) -> Self {
        Self {
            user_input: user_input.clone(),
            result_holder: Vec::new(),
        }
    }
}

pub struct ResultHandler {
    pub result_sender: flume::Sender<ResultMsg>,
    result_receiver: flume::Receiver<ResultMsg>,

    holder_map: HashMap<i32, ResultHolder>,
}

impl ResultHandler {
    pub fn new() -> Self {
        let (result_sender, result_receiver) = flume::unbounded();

        ResultHandler {
            result_sender,
            result_receiver,
            holder_map: Default::default(),
        }
    }

    pub async fn accept_messages(&mut self) {
        loop {
            match self.result_receiver.recv_async().await {
                Ok(msg) => match msg {
                    ResultMsg::Result(input, mut results) => {
                        match self.holder_map.get_mut(&input.window_id) {
                            None => {
                                error!("Unable to append result to {:?}", input);
                            }
                            Some(holder) => {
                                if holder.user_input.as_ref() == input.as_ref() {
                                    holder.result_holder.append(&mut results)
                                }
                            }
                        }
                    }
                    ResultMsg::UserInput(input) => {
                        match self.holder_map.get_mut(&input.window_id) {
                            None => {
                                self.holder_map.insert(
                                    input.window_id.clone(),
                                    ResultHolder::new(input.clone()),
                                );
                            }
                            Some(holder) => {
                                holder.user_input = input.clone();
                                holder.result_holder.clear()
                            }
                        }
                    }
                    ResultMsg::RemoveWindow(window_id) => {
                        self.holder_map.remove(&window_id);
                    }
                },
                Err(ex) => {
                    error!("unable to receive message: {:?}", ex);
                }
            }
        }
    }

    pub fn start() {
        thread::spawn(move || {
            let mut result_handler = Self::new();
            block_on(result_handler.accept_messages());
        });
    }
}
