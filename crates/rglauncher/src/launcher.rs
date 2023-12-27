use std::sync::Arc;

use crate::arguments::Arguments;
use crate::window::RGWindow;
use backend::plugindispatcher::{DispatchMsg, PluginDispatcher};
use flume::{Receiver, Sender};
use gio::prelude::*;
use glib::MainContext;
use gtk::prelude::*;
use gtk::Application;

#[derive(Clone)]
pub struct Launcher {
    app: Application,
    app_args: Arc<Arguments>,

    dispatcher_sender: flume::Sender<DispatchMsg>,

    pub launcher_sender: Sender<LauncherMsg>,
    launcher_receiver: Receiver<LauncherMsg>,
}

pub enum LauncherMsg {
    SelectSomething,
    Exit,
    NewWindow,
}

impl Launcher {
    pub fn new(
        application: Application,
        arguments: Arc<Arguments>,
        launcher_sender: Sender<LauncherMsg>,
        launcher_receiver: Receiver<LauncherMsg>,
    ) -> Self {
        let dispatch_sender =
            PluginDispatcher::start(arguments.dict_dir.as_str(), arguments.clip_db.as_str());

        Launcher {
            app: application,
            app_args: arguments,

            dispatcher_sender: dispatch_sender,

            launcher_sender,
            launcher_receiver,
        }
    }

    pub fn new_window(&self) -> RGWindow {
        let window = RGWindow::new(&self.app, 
            self.app_args.clone(),
            &self.dispatcher_sender);
        let win = window.clone();

        let launcher_receiver = self.launcher_receiver.clone();
        let dispatcher_sender = self.dispatcher_sender.clone();

        MainContext::ref_thread_default().spawn_local(async move {
            loop {
                match launcher_receiver.recv_async().await {
                    Ok(msg) => match msg {
                        LauncherMsg::Exit => {}
                        LauncherMsg::NewWindow => {
                            dispatcher_sender
                                .send(DispatchMsg::RefreshContent)
                                .expect("");
                            win.show_window();
                        }
                        LauncherMsg::SelectSomething => {
                            win.hide_window();
                        }
                    },
                    Err(_) => {}
                }
            }
        });

        window
    }
}
