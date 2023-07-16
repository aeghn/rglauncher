use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::task::ready;
use futures::future::{Abortable, Aborted, AbortHandle};
use gio::ListStore;
use glib::{BoxedAnyObject, Continue, MainContext, Receiver, Sender};
use gtk::ResponseType::No;
use tracing::error;
use crate::plugins::app::AppPlugin;
use crate::plugins::clipboard::ClipboardPlugin;
use crate::plugins::{Plugin, PluginResult};
use crate::plugins::windows::HyprWindows;
use crate::shared::UserInput;

pub struct PluginHandler {
    plugin: Box<dyn Plugin>,
    abort_handle: RefCell<Option<AbortHandle>>
}

impl PluginHandler {
    fn new(plugin: Box<dyn Plugin>,  abort_handle: RefCell<Option<AbortHandle>>) -> Self {
        PluginHandler {
            plugin,
            abort_handle,
        }
    }

    async fn handle_message(&self, user_input: UserInput) -> (UserInput, Vec<Box<dyn PluginResult>>) {
        (user_input.clone(), self.plugin.handle_input(&user_input))
    }

    fn replace_abort(&mut self, abort_handle: AbortHandle) -> Option<AbortHandle> {
        self.abort_handle.get_mut().replace(abort_handle)
    }
}

pub struct Dispatcher {
    plugins: Vec<Arc<Mutex<PluginHandler>>>,
    result_sender: Sender<(UserInput, Vec<Box<dyn PluginResult>>)>
}

impl Dispatcher {
    pub fn new(result_sender: Sender<(UserInput, Vec<Box<dyn PluginResult>>)>) -> Self {
        let app_plugin = AppPlugin::new();
        let window_plugin = HyprWindows::new();
        let clip_plugin = ClipboardPlugin::new(crate::constant::STORE_DB);

        let mut plugins : Vec<Arc<Mutex<PluginHandler>>> = vec![];

        plugins.push(Arc::new(Mutex::new( PluginHandler::new(Box::new(app_plugin), RefCell::new(None)))));
        plugins.push(Arc::new(Mutex::new(PluginHandler::new(Box::new(window_plugin), RefCell::new(None)))));
        plugins.push(Arc::new(Mutex::new(PluginHandler::new(Box::new(clip_plugin), RefCell::new(None)))));


        Dispatcher {
            plugins,
            result_sender
        }
    }

    pub fn handle_messages(&self, user_input: UserInput) {
            self.plugins.iter().for_each(|plugin| {
                let (abort_handle, abort_registration) = AbortHandle::new_pair();

                // if let Some(handle) = plugin.replace_abort(abort_handle) {
                //     handle.abort();
                // }

                let query_info_fut =
                    Abortable::new(, abort_registration);

                {
                    let sender = self.result_sender.clone();
                    MainContext::ref_thread_default().spawn_local(async move {
                        match query_info_fut.await {
                            Ok(res) => {
                                sender.send(res).unwrap();
                            }
                            Err(_) => {}
                        }
                    });
                }
        });
    }
}