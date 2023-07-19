use std::sync::{Arc, Mutex};


use futures::future::{Abortable, AbortHandle};
use glib::{MainContext, PRIORITY_DEFAULT};
use crate::plugins::{Plugin, PluginResult};

use tracing::error;
use crate::shared::UserInput;
use crate::sidebar::SidebarMsg;

#[derive(Debug)]
pub enum PluginMessage {
    Input(String)
}


pub struct PluginWorker<P: Plugin> {
    plugin: Arc<Mutex<P>>,
    abort_handle: Option<AbortHandle>,
    receiver: flume::Receiver<PluginMessage>,
    result_sender: flume::Sender<SidebarMsg>
}


async fn handle_message<P: Plugin>(plugin: Arc<Mutex<P>>, _input: UserInput) -> Option<SidebarMsg> {
    let _p = plugin.lock().unwrap();
    let pr = _p.handle_input(&_input);
    Some(SidebarMsg::PluginResult(_input, pr))
}

impl <P: Plugin + 'static> PluginWorker<P> {
    pub fn new(plugin: P,
               receiver: flume::Receiver<PluginMessage>,
               result_sender: flume::Sender<SidebarMsg>) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            abort_handle: None,
            receiver,
            result_sender,
        }
    }

    pub fn launch(result_sender: &flume::Sender<SidebarMsg>,
                  plugin: impl Plugin + 'static,
                  receiver: &flume::Receiver<PluginMessage>) {
        let result_sender = result_sender.clone();
        let receiver = receiver.clone();
        MainContext::ref_thread_default().spawn_local_with_priority(
            PRIORITY_DEFAULT,
            async move {
                let mut plugin_worker =
                    PluginWorker::new(plugin,
                                      receiver,
                                      result_sender);
                plugin_worker.run().await;
            });
    }

    pub async fn run(&mut self) {
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