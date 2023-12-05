use std::marker::PhantomData;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use flume::Sender;

use futures::StreamExt;
use gio::prelude::CancellableExt;
use gio::Cancellable;
use glib::{idle_add, ControlFlow, MainContext};
use tracing::info;

use crate::inputbar::InputMessage;
use crate::inputbar::InputMessage::{RefreshContent, TextChanged};
use crate::plugins::{Plugin, PluginResult};
use crate::userinput::UserInput;

use crate::sidebar::SidebarMsg;

pub struct PluginWorker<P: Plugin<R>, R: PluginResult> {
    plugin: Arc<Mutex<P>>,
    idle_workers: Arc<AtomicU8>,
    results: Arc<Mutex<Vec<R>>>,
    cancelable: Option<Cancellable>,
    receiver: async_broadcast::Receiver<Arc<InputMessage>>,
    result_sender: Sender<SidebarMsg>,
    phantom_r: PhantomData<R>,
}

impl<P: Plugin<R> + 'static + Send, R: PluginResult + 'static> PluginWorker<P, R> {
    pub fn new(
        plugin: P,
        input_receiver: async_broadcast::Receiver<Arc<InputMessage>>,
        result_sender: Sender<SidebarMsg>,
    ) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            idle_workers: Arc::new(AtomicU8::new(0)),
            results: Arc::new(Mutex::new(Vec::<R>::new())),
            cancelable: None,
            receiver: input_receiver,
            result_sender,
            phantom_r: Default::default(),
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

                match msg {
                    TextChanged(input) => {
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

                                Some(result)
                            } else {
                                None
                            };

                            vr
                        });

                        if let Ok(Some(Ok(vec))) = jh.await {
                            let result_arc = self.results.clone();
                            {
                                let mut op = result_arc.lock().unwrap();
                                *op = vec;
                            }

                            let idle_worker = self.idle_workers.clone();
                            if idle_worker.fetch_add(1, Ordering::SeqCst) > 1 {
                                idle_worker.fetch_sub(1, Ordering::SeqCst);
                            } else {
                                let sender = self.result_sender.clone();
                                idle_add(move || match result_arc.lock() {
                                    Ok(mut vec_guard) => {
                                        if let Some(vv) = vec_guard.pop() {
                                            sender
                                                .send(SidebarMsg::PluginResult(
                                                    ui.clone(),
                                                    Box::new(vv) as Box<dyn PluginResult>,
                                                ))
                                                .unwrap();
                                            ControlFlow::Continue
                                        } else {
                                            idle_worker.fetch_sub(1, Ordering::SeqCst);
                                            ControlFlow::Break
                                        }
                                    }
                                    _ => {
                                        idle_worker.fetch_sub(1, Ordering::SeqCst);
                                        ControlFlow::Break
                                    }
                                });
                            }
                        }
                    }
                    RefreshContent => {
                        info!("begin to refresh windows");
                        let plugin_arc = self.plugin.clone();
                        gio::spawn_blocking(move || {
                            if let Ok(mut lock) = plugin_arc.lock() {
                                lock.refresh_content();
                            }
                        });
                    }

                    _ => {}
                }
            }
        }
    }
}
