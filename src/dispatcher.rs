use std::sync::{Arc, Mutex};
use std::task::ready;
use gio::ListStore;
use glib::{BoxedAnyObject, Continue, MainContext, Receiver, Sender};
use gtk::ResponseType::No;
use tracing::error;
use crate::plugins::app::AppPlugin;
use crate::plugins::clipboard::ClipboardPlugin;
use crate::plugins::{Plugin, PluginResult};
use crate::plugins::windows::HyprWindows;
use crate::shared::UserInput;

pub struct Dispatcher {
    plugins: Vec<Box<dyn Plugin + Send>>,
    receiver: Receiver<UserInput>,
    result_sender: Sender<Vec<Box<dyn PluginResult + Send>>>
}

impl Dispatcher {
    pub fn new(receiver: Receiver<UserInput>,
               result_sender: Sender<Vec<Box<dyn PluginResult + Send>>>) -> Self {
        let app_plugin = AppPlugin::new();
        let window_plugin = HyprWindows::new();
        let clip_plugin = ClipboardPlugin::new("/home/chin/.cache/rglauncher/store.db");

        let mut plugins : Vec<Box<dyn Plugin + Send>> = vec![];

        plugins.push(Box::new(app_plugin));
        plugins.push(Box::new(window_plugin));
        plugins.push(Box::new(clip_plugin));

        Dispatcher {
            plugins,
            receiver,
            result_sender
        }
    }

    pub fn handle_messages(&self) {
        self.receiver.attach(None, |_user_input| {
            tokio::spawn(async move {
                let mut tmp_vec: Vec<Box<dyn PluginResult + Send>> = vec![];
                self.plugins.iter().for_each(|p| {
                    let mut v = p.handle_input(&_user_input);
                    for vv in v {
                        tmp_vec.push(vv);
                    }
                });

                self.result_sender.send(tmp_vec).unwrap();
            });

            Continue(true)
        });
    }
}