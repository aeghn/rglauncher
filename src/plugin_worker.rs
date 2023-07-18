use std::sync::{Arc, Mutex};
use std::task::Poll;
use flume::{RecvError, TryRecvError};
use futures::future::{Abortable, Aborted, AbortHandle};
use futures::Stream;
use glib::{Error, MainContext};
use crate::plugins::{Plugin, PluginResult};
use gio::FileInfo;
use crate::shared::UserInput;

pub enum PluginMessage {
    Input(String)
}

pub enum PluginOutputMessage {
    Output(Vec<FileInfo>)
}

pub struct PluginWorker<P: Plugin> {
    plugin: Arc<Mutex<P>>,
    abort_handle: Option<AbortHandle>,
    receiver: flume::Receiver<PluginMessage>,
    result_sender: flume::Sender<PluginOutputMessage>
}


async fn handle_message<P: Plugin>(plugin: Arc<Mutex<P>>, input: UserInput) -> Option<Vec<FileInfo>> {
    let p = plugin.lock().unwrap();
    // Some(p.handle_input(&input))
    let mut  vec = vec![];
    vec.push(FileInfo::new());
    Some(vec)
}

impl <P: Plugin + 'static> PluginWorker<P> {
    pub fn new(plugin: P, receiver: flume::Receiver<PluginMessage>,
               result_sender: flume::Sender<PluginOutputMessage>) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            abort_handle: None,
            receiver,
            result_sender,
        }
    }

    pub fn launch(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.try_recv() {
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
                                        sender.send(PluginOutputMessage::Output(rs)).unwrap()
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