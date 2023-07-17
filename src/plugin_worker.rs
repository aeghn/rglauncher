use std::sync::{Arc, Mutex};
use std::task::Poll;
use flume::{RecvError, TryRecvError};
use futures::future::{Abortable, Aborted, AbortHandle};
use futures::Stream;
use glib::{Error, MainContext};
use gtk::ResponseType::No;
use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;

pub enum PluginMessage {
    Input(String)
}

pub struct PluginWorker<P: Plugin> {
    plugin:  Arc<Mutex<P>>,
    abort_handle: Option<AbortHandle>,
    receiver: flume::Receiver<PluginMessage>,
    result_sender: glib::Sender<Vec<Box<dyn PluginResult>>>
}


async fn handle_message<P: Plugin>(plugin: Arc<Mutex<P>>, input: UserInput) -> Option<Vec<Box<dyn PluginResult>>> {
    let p = plugin.lock().unwrap();
    Some(p.handle_input(&input))
}

impl <P: Plugin + 'static> PluginWorker<P> {
    pub fn launch(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(msg) => {
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
                                match query_info_fut.await {
                                    Ok(r) => {
                                        match r {
                                            None => {}
                                            Some(rs) => {
                                                sender.send(rs).unwrap()
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            });
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }
}