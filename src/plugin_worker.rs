use std::borrow::Borrow;
use std::cell::RefCell;
use std::mem::take;
use std::rc::Rc;
use std::sync::{Arc, LockResult, Mutex};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;
use flume::{Receiver, Sender};


use futures::future::{Abortable, AbortHandle};
use futures::StreamExt;
use gio::{Cancellable, JoinHandle, Task};
use gio::prelude::CancellableExt;
use glib::{BoxedAnyObject, Continue, idle_add, MainContext, PRIORITY_DEFAULT, PRIORITY_DEFAULT_IDLE, PRIORITY_HIGH_IDLE, StaticType, ToValue, Type, Value};
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
    idle_workers: Arc<AtomicU8>,
    results: Arc<Mutex<Vec<R>>>,
    cancelable: Option<Cancellable>,
    receiver: async_broadcast::Receiver<Arc<InputMessage>>,
    result_sender: Sender<SidebarMsg>
}

impl <P: Plugin<R> + 'static + Send, R: PluginResult + 'static> PluginWorker<P, R> {
    pub fn new(plugin: P,
               input_receiver: async_broadcast::Receiver<Arc<InputMessage>>,
               result_sender: Sender<SidebarMsg>) -> Self {
        PluginWorker {
            plugin: Arc::new(Mutex::new(plugin)),
            idle_workers: Arc::new(AtomicU8::new(0)),
            results: Arc::new(Mutex::new(Vec::<R>::new())),
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

                            Some(result)
                        } else {
                            None
                        };

                        vr
                    });

                    if let Ok(Some(vec)) = jh.await {
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
                            idle_add(move || {
                                match result_arc.lock() {
                                    Ok(mut vec_guard) => {
                                        if let Some(vv) = vec_guard.pop() {
                                            sender.send(SidebarMsg::PluginResult(
                                                ui.clone(),
                                                Box::new(vv) as Box<dyn PluginResult>))
                                                .unwrap();
                                            Continue(true)
                                        } else {
                                            idle_worker.fetch_sub(1, Ordering::SeqCst);
                                            Continue(false)
                                        }
                                    }
                                    _ => {
                                        idle_worker.fetch_sub(1, Ordering::SeqCst);
                                        Continue(false)
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
    }
}