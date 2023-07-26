use std::borrow::Borrow;
use std::mem::take;
use std::sync::{Arc, LockResult, Mutex};
use std::thread::sleep;
use flume::{Receiver, Sender};


use futures::future::{Abortable, AbortHandle};
use futures::StreamExt;
use gio::{Cancellable, JoinHandle, Task};
use gio::prelude::CancellableExt;
use glib::{BoxedAnyObject, MainContext, PRIORITY_DEFAULT, PRIORITY_DEFAULT_IDLE, PRIORITY_HIGH_IDLE, StaticType, ToValue, Type, Value};
use glib::ffi::G_PRIORITY_HIGH_IDLE;
use glib::value::{FromValue, ValueType};
use gtk::ResponseType::No;
use tracing::error;
use crate::inputbar::InputMessage;
use crate::inputbar::InputMessage::TextChanged;
use crate::plugins::{Plugin, PluginResult};


use crate::shared::UserInput;
use crate::sidebar::SidebarMsg;


pub struct PluginWorker<P: Plugin<R>, R: PluginResult> {
    plugin: Arc<Mutex<P>>,
    results: Option<R>,
    cancelable: Option<JoinHandle<()>>,
    receiver: async_broadcast::Receiver<Arc<InputMessage>>,
    result_sender: Sender<SidebarMsg>
}



impl <P: Plugin<R> + 'static + Send, R: PluginResult + 'static> PluginWorker<P, R> {
    pub fn new(plugin: P,
               input_receiver: async_broadcast::Receiver<Arc<InputMessage>>,
               result_sender: Sender<SidebarMsg>) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            results: None,
            cancelable: None,
            receiver: input_receiver,
            result_sender,
        }
    }

    pub fn launch(result_sender: &Sender<SidebarMsg>,
                  plugin: impl Plugin<R> + 'static + Send,
                  input_receiver: &async_broadcast::Receiver<Arc<InputMessage>>) {
        let result_sender = result_sender.clone();
        let input_receiver = input_receiver.clone();
        MainContext::ref_thread_default().spawn_local_with_priority(
            PRIORITY_DEFAULT,
            async move {
                let mut plugin_worker =
                    PluginWorker::new(plugin, input_receiver, result_sender);
                plugin_worker.loop_recv().await;
            });
    }

    async fn loop_recv(&mut self) {
        loop {
            let pn = self.receiver.next().await;
            let rs = self.result_sender.clone();
            if let Some(msg) = pn {
                let msg: InputMessage = msg.as_ref().clone();
                match msg {
                    TextChanged(input) => {
                        let plugin_arc = self.plugin.clone();
                        let ui = UserInput::new(input.as_str());
                        let jh = gio::spawn_blocking( move || {
                            let vr = if let Ok(lock) = plugin_arc.lock() {
                                let ui = UserInput::new(input.as_str());
                                let result = lock.handle_input(&ui);
                                let res = result.into_iter().map(|r|
                                    Box::new(r ) as Box<dyn PluginResult>).collect();
                                res
                            } else {
                                vec![]
                            };
                            vr
                        });
                        jh.()
                    }
                    _ => {}
                }
            }
        }
    }
}