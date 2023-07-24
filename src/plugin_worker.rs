use std::borrow::Borrow;
use std::mem::take;
use std::sync::Arc;
use tokio::sync::{Mutex};
use std::thread::sleep;
use flume::{Receiver, Sender};


use futures::future::{Abortable, AbortHandle};
use gio::{Cancellable, Task};
use gio::prelude::CancellableExt;
use glib::{BoxedAnyObject, MainContext, PRIORITY_DEFAULT, PRIORITY_HIGH_IDLE, StaticType, ToValue, Type, Value};
use glib::ffi::G_PRIORITY_HIGH_IDLE;
use glib::value::{FromValue, ValueType};
use gtk::ResponseType::No;
use tokio::task::JoinHandle;
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
    receiver: barrage::Receiver<InputMessage>,
    result_sender: Sender<SidebarMsg>
}



impl <P: Plugin<R> + 'static + Send, R: PluginResult + 'static> PluginWorker<P, R> {
    pub fn new(plugin: P,
               input_receiver: barrage::Receiver<InputMessage>,
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
                  input_receiver: &barrage::Receiver<InputMessage>) {
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
            let pn = self.receiver.recv_async().await;
            if let Some(plugin) = self.cancelable.take() {
                plugin.abort();
            }

            if let Ok(msg) = pn {
                match msg {
                    TextChanged(input) => {
                        let plugin_arc = self.plugin.clone();
                        let result_sender = self.result_sender.clone();
                        let fu = tokio::spawn(async move {
                            let p = plugin_arc.clone();
                            if let lock = p.lock().await {
                                let ui = UserInput::new(input.as_str());
                                let result = lock.handle_input(&ui);

                                for x in result {
                                    let rs = result_sender.clone();
                                    let ui = ui.clone();
                                    MainContext::default().invoke_with_priority(
                                        PRIORITY_HIGH_IDLE,
                                        move|| {
                                            rs.send(SidebarMsg::PluginResult(
                                                ui.clone(),
                                                Box::new(x)
                                            )).unwrap();
                                        }
                                    );
                                }
                            };
                        });
                        self.cancelable.replace(fu);
                    }
                    _ => {}
                }
            }
        }
    }
}