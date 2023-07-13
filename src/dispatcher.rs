use std::sync::{Arc, Mutex};
use std::task::ready;
use futures::future::{Abortable, AbortHandle};
use gio::ListStore;
use glib::{BoxedAnyObject, Continue, MainContext, Receiver, Sender};
use gtk::ResponseType::No;
use tracing::error;
use crate::plugins::app::AppPlugin;
use crate::plugins::clipboard::ClipboardPlugin;
use crate::plugins::{Plugin, PluginResult};
use crate::plugins::windows::HyprWindows;
use crate::shared::UserInput;

async fn plugin_handle(&plugin: Box<dyn Plugin>,  sender: Sender<Vec<Box<dyn PluginResult>>>) {
    plugin.handle_input();
}

pub struct Dispatcher {
    plugins: Vec<Box<dyn Plugin>>,
    receiver: Receiver<UserInput>,
    result_sender: Sender<Vec<Box<dyn PluginResult>>>
}

impl Dispatcher {
    pub fn new(receiver: Receiver<UserInput>,
               result_sender: Sender<Vec<Box<dyn PluginResult>>>) -> Self {
        let app_plugin = AppPlugin::new();
        let window_plugin = HyprWindows::new();
        let clip_plugin = ClipboardPlugin::new(crate::constant::STORE_DB);

        let mut plugins : Vec<Box<dyn Plugin>> = vec![];

        plugins.push(Box::new(app_plugin));
        plugins.push(Box::new(window_plugin));
        plugins.push(Box::new(clip_plugin));

        Dispatcher {
            plugins,
            receiver,
            result_sender
        }
    }



    pub fn handle_messages(&mut self) {
        let sender = self.result_sender.clone();
        self.receiver.attach(None, |_user_input| {
            MainContext::ref_thread_default().spawn_local(async move {
                let mut tmp_vec: Vec<Box<dyn PluginResult>> = vec![];
                self.plugins.iter().for_each(|p| {
                    let mut v = p.handle_input(&_user_input);
                    for vv in v {
                        tmp_vec.push(vv);
                    }
                });

                let (abort_handle, abort_registration) = AbortHandle::new_pair();

                if let Some(handle) = self.abort_preview.replace(abort_handle) {
                    handle.abort();
                }

                let query_info_fut =
                    Abortable::new(, abort_registration);

                sender.send(tmp_vec).unwrap();
            });

            Continue(true)
        });
    }
}