use std::sync::{Arc, Mutex};


use futures::future::{Abortable, AbortHandle};
use glib::{MainContext};
use crate::plugins::{Plugin, PluginResult};

use tracing::error;
use crate::shared::UserInput;

#[derive(Debug)]
pub enum PluginMessage {
    Input(String)
}


pub struct PluginWorker<P: Plugin> {
    plugin: Arc<Mutex<P>>,
    abort_handle: Option<AbortHandle>,
    receiver: flume::Receiver<PluginMessage>,
    result_sender: flume::Sender<Vec<Box<dyn PluginResult>>>
}


async fn handle_message<P: Plugin>(plugin: Arc<Mutex<P>>, _input: UserInput) -> Option<Vec<Box<dyn PluginResult>>> {
    let _p = plugin.lock().unwrap();
    // Some(p.handle_input(&input));
    let vec = vec![];
    Some(vec)
}

impl <P: Plugin + 'static> PluginWorker<P> {
    pub fn new(plugin: P, receiver: flume::Receiver<PluginMessage>,
               result_sender: flume::Sender<Vec<Box<dyn PluginResult>>>) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            abort_handle: None,
            receiver,
            result_sender,
        }
    }

    pub async fn launch(&mut self) {
        loop {
            let pn = self.receiver.recv_async().await;
            if let Ok(msg) = pn {
                error!("plugin worker got message: {:?}", msg);
                match msg {
                    PluginMessage::Input(input) => {
                        let (abort_handle, abort_registration) = AbortHandle::new_pair();

                        if let Some(handle) = self.abort_handle.replace(abort_handle) {
                            handle.abort();
                        }

                        let ui = UserInput { input };
                        let query_info_fut =
                            Abortable::new(handle_message(Arc::clone(&self.plugin), ui),
                                           abort_registration);

                        let sender = self.result_sender.clone();
                        MainContext::ref_thread_default().spawn_local(async move {
                            if let Ok(r) = query_info_fut.await {
                                match r {
                                    None => {}
                                    Some(rs) => {
                                        sender.send(rs).unwrap();
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}