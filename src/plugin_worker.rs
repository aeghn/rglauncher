use std::sync::{Arc, Mutex};


use futures::future::{Abortable, AbortHandle};
use glib::{MainContext};
use crate::plugins::{Plugin};

use tracing::error;
use crate::plugins::clipboard::ClipPluginResult;
use crate::shared::UserInput;

pub enum PluginMessage {
    Input(String)
}


pub struct PluginWorker<P: Plugin> {
    plugin: Arc<Mutex<P>>,
    abort_handle: Option<AbortHandle>,
    receiver: flume::Receiver<PluginMessage>,
    result_sender: flume::Sender<Vec<ClipPluginResult>>
}


async fn handle_message<P: Plugin>(plugin: Arc<Mutex<P>>, _input: UserInput) -> Option<Vec<ClipPluginResult>> {
    let _p = plugin.lock().unwrap();
    // Some(p.handle_input(&input));
    let vec = vec![];
    Some(vec)
}

impl <P: Plugin + 'static> PluginWorker<P> {
    pub fn new(plugin: P, receiver: flume::Receiver<PluginMessage>,
               result_sender: flume::Sender<Vec<ClipPluginResult>>) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            abort_handle: None,
            receiver,
            result_sender,
        }
    }

    pub async fn launch(&mut self) {
        error!("=============");
        error!("..............");

        loop {
            let pn = self.receiver.recv_async().await;


            if let Ok(msg) = pn {
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
                                        sender.send(rs).unwrap()
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