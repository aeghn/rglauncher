use std::sync::Arc;

use crate::application::RGLApplication;
use crate::window::RGWindow;
use flume::{Receiver, Sender};
use glib::MainContext;
use rglcore::{config::Config, plugindispatcher::{DispatchMsg, PluginDispatcher}};

#[derive(Clone, Debug)]
pub struct Launcher {
    app: RGLApplication,
    pub config: Arc<Config>,

    dispatcher_tx: Sender<DispatchMsg>,

    pub launcher_tx: Sender<LauncherMsg>,
    launcher_rx: Receiver<LauncherMsg>,
}

pub enum LauncherMsg {
    SelectSomething,
    Exit,
    NewWindow,
}

impl Launcher {
    pub fn new(
        application: RGLApplication,
        config: Arc<Config>,
        launcher_tx: &Sender<LauncherMsg>,
        launcher_rx: &Receiver<LauncherMsg>,
    ) -> Self {
        let dispatcher_tx = PluginDispatcher::start(&config);

        Launcher {
            app: application,
            config,

            dispatcher_tx,

            launcher_tx: launcher_tx.clone(),
            launcher_rx: launcher_rx.clone(),
        }
    }

    pub fn new_window(&self) {
        // let win = window.clone();

        let launcher_rx = self.launcher_rx.clone();
        let launcher_tx = self.launcher_tx.clone();
        let dispatcher_tx = self.dispatcher_tx.clone();
        let app_args = self.config.clone();
        let app = self.app.clone();

        RGWindow::setup_one(&app, app_args.clone(), &dispatcher_tx, &launcher_tx);

        MainContext::ref_thread_default().spawn_local(async move {
            let dispatcher_tx = dispatcher_tx.clone();
            let launcher_tx = launcher_tx.clone();
            let app_args = app_args.clone();
            let app = app.clone();
            loop {
                match launcher_rx.recv_async().await {
                    Ok(msg) => match msg {
                        LauncherMsg::Exit => {}
                        LauncherMsg::NewWindow => {
                            dispatcher_tx.send(DispatchMsg::RefreshContent).expect("");
                            RGWindow::setup_one(
                                &app,
                                app_args.clone(),
                                &dispatcher_tx,
                                &launcher_tx,
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
