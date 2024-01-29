use std::sync::Arc;

use crate::application::RGLApplication;
use crate::arguments::Arguments;
use crate::window::RGWindow;
use backend::plugindispatcher::{DispatchMsg, PluginDispatcher};
use flume::{Receiver, Sender};
use glib::MainContext;

#[derive(Clone, Debug)]
pub struct Launcher {
    app: RGLApplication,
    pub app_args: Arc<Arguments>,

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
        application: RGLApplication,
        arguments: Arc<Arguments>,
        launcher_sender: Sender<LauncherMsg>,
        launcher_receiver: Receiver<LauncherMsg>,
    ) -> Self {
        let dispatcher_sender =
            PluginDispatcher::start(arguments.dict_dir.as_str(), arguments.clip_db.as_str());

        Launcher {
            app: application,
            app_args: arguments,

            dispatcher_sender,

            launcher_sender,
            launcher_receiver,
        }
    }

    pub fn new_window(&self) {
        // let win = window.clone();

        let launcher_receiver = self.launcher_receiver.clone();
        let launcher_sender = self.launcher_sender.clone();
        let dispatcher_sender = self.dispatcher_sender.clone();
        let app_args = self.app_args.clone();
        let app = self.app.clone();

        RGWindow::setup_one(&app, app_args.clone(), &dispatcher_sender, &launcher_sender);

        MainContext::ref_thread_default().spawn_local(async move {
            let dispatcher_sender = dispatcher_sender.clone();
            let launcher_sender = launcher_sender.clone();
            let app_args = app_args.clone();
            let app = app.clone();
            loop {
                match launcher_receiver.recv_async().await {
                    Ok(msg) => match msg {
                        LauncherMsg::Exit => {}
                        LauncherMsg::NewWindow => {
                            dispatcher_sender
                                .send(DispatchMsg::RefreshContent)
                                .expect("");
                            RGWindow::setup_one(
                                &app,
                                app_args.clone(),
                                &dispatcher_sender,
                                &launcher_sender,
                            );
                        }
                        LauncherMsg::SelectSomething => {
                            // win.hide_window();
                        }
                    },
                    Err(_) => {}
                }
            }
        });
    }
}
