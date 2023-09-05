use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use flume::Sender;

use futures::StreamExt;
use gio::prelude::CancellableExt;
use gio::Cancellable;
use glib::{idle_add, ControlFlow, MainContext, BoxedAnyObject, idle_add_local, idle_add_local_once};

use crate::inputbar::InputMessage;
use crate::inputbar::InputMessage::TextChanged;
use crate::plugins::{Plugin, PluginResult};
use crate::user_input::UserInput;

use crate::sidebar::SidebarMsg;
use crate::sidebar::SidebarMsg::PluginResults;

pub struct PluginWorker<P: Plugin<R>, R: PluginResult> {
    plugin: Arc<Mutex<P>>,
    cancelable: Option<Cancellable>,
    receiver: async_broadcast::Receiver<Arc<InputMessage>>,
    result_sender: Sender<SidebarMsg>,
}

impl<P: Plugin<R> + 'static + Send, R: PluginResult + 'static> PluginWorker<P, R> {
    pub fn new(
        plugin: P,
        input_receiver: async_broadcast::Receiver<Arc<InputMessage>>,
        result_sender: Sender<SidebarMsg>,
    ) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            cancelable: None,
            receiver: input_receiver,
            result_sender,
        }
    }

    pub fn launch<F>(
        result_sender: &Sender<SidebarMsg>,
        plugin_builder: F,
        input_receiver: &async_broadcast::Receiver<Arc<InputMessage>>,
    ) where
        F: Fn() -> P + 'static,
    {
        let result_sender = result_sender.clone();
        let input_receiver = input_receiver.clone();
        MainContext::ref_thread_default().spawn_local_with_priority(
            glib::source::Priority::DEFAULT_IDLE,
            async move {
                let plugin = plugin_builder();
                let mut plugin_worker = PluginWorker::new(plugin, input_receiver, result_sender);
                plugin_worker.loop_recv().await;
            },
        );
    }

    async fn loop_recv(&mut self) {
        loop {
            let pn = self.receiver.next().await;
            if let Some(msg) = pn {
                let msg: InputMessage = msg.as_ref().clone();
                if let TextChanged(input) = msg {
                    let plugin_arc = self.plugin.clone();
                    let ui = UserInput::new(input.as_str());
                    let cancellable = Cancellable::new();

                    let cancelable_receiver = cancellable.clone();
                    if let Some(cc) = self.cancelable.replace(cancellable) {
                        cc.cancel();
                    }

                    let ui_clone = ui.clone();
                    let jh = gio::spawn_blocking(move || {
                        let vr = if let Ok(lock) = plugin_arc.lock() {
                            if cancelable_receiver.is_cancelled() {
                                return None;
                            }

                            let result = lock.handle_input(&ui_clone);
                            if cancelable_receiver.is_cancelled() {
                                return None;
                            }
                            let converted_result: Vec<Box<dyn PluginResult>> =
                                result.into_iter().map(|r| Box::new(r) as Box<dyn PluginResult>)
                                    .collect();
                            Some(converted_result)
                        } else {
                            None
                        };

                        vr
                    });

                    if let Ok(Some(vec)) = jh.await {
                        let result_sender = self.result_sender.clone();
                        idle_add_local_once(move || {
                            result_sender.send(PluginResults(ui, vec)).unwrap();
                        });
                    }
                }
            }
        }
    }
}
