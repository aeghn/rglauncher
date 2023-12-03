use flume::{Receiver, Sender};
use glib::{clone, BoxedAnyObject, MainContext};
use std::sync::{Arc, RwLock};
use futures::io::Window;

use gio::prelude::*;
use gtk::{Application, gdk};
use gtk::prelude::*;
use gtk::{
    self,
    traits::{BoxExt, GtkWindowExt, WidgetExt},
    ApplicationWindow, Entry,
};
use gtk::ResponseType::No;

use tracing::error;
use webkit6::prelude::WebsocketConnectionExtManual;
use crate::arguments;
use crate::arguments::Arguments;

use crate::inputbar::{InputBar, InputMessage};
use crate::plugin_worker::PluginWorker;
use crate::plugins::app::{AppPlugin, AppResult};
use crate::plugins::calculator::{CalcResult, Calculator};
use crate::plugins::clipboard::{ClipPluginResult, ClipboardPlugin};
use crate::plugins::dict::{DictPlugin, DictPluginResult};
use crate::plugins::windows::{HyprWindowResult, HyprWindows};
use crate::preview::Preview;

use crate::sidebar::SidebarMsg;
use crate::window::RGWindow;

#[derive(Clone)]
pub struct Launcher {
    app: Application,
    app_args: Arguments,

    app_msg_sender: Sender<AppMsg>,
    app_msg_receiver: Receiver<AppMsg>,
    input_sender: async_broadcast::Sender<Arc<InputMessage>>,
    input_receiver: async_broadcast::Receiver<Arc<InputMessage>>,
    selection_change_sender: Sender<BoxedAnyObject>,
    selection_change_receiver: Receiver<BoxedAnyObject>,
    sidebar_sender: Sender<SidebarMsg>,
    sidebar_receiver: Receiver<SidebarMsg>,

    db: Arc<RwLock<Option<rusqlite::Connection>>>,

    pub window: Option<RGWindow>
}

pub enum AppMsg {
    Exit,
}

impl Launcher {
    pub fn new(application: Application, arguments: Arguments) -> Self {
        let (mut input_sender, input_receiver) = async_broadcast::broadcast(1);
        input_sender.set_overflow(true);

        let (selection_change_sender, selection_change_receiver) = flume::unbounded();
        let (sidebar_sender, sidebar_receiver) = flume::unbounded();
        let (app_msg_sender, app_msg_receiver) = flume::unbounded();

        Launcher {
            app: application,
            app_args: arguments,

            app_msg_sender,
            app_msg_receiver,
            input_sender,
            input_receiver,
            selection_change_sender,
            selection_change_receiver,
            sidebar_sender,
            sidebar_receiver,

            db: Arc::new(RwLock::default()),

            window: None
        }
    }

    pub fn launch_plugins(&self) {
        let sidebar_sender = self.sidebar_sender.clone();
        let input_broadcast = self.input_receiver.clone();

        let clip_db = self.app_args.clip_db.clone();
        PluginWorker::<ClipboardPlugin, ClipPluginResult>::launch(
            &sidebar_sender,
            move || ClipboardPlugin::new(clip_db.as_str()),
            &input_broadcast,
        );

        PluginWorker::<AppPlugin, AppResult>::launch(
            &sidebar_sender,
            || AppPlugin::new(),
            &input_broadcast,
        );

        PluginWorker::<HyprWindows, HyprWindowResult>::launch(
            &sidebar_sender,
            || HyprWindows::new(),
            &input_broadcast,
        );

        let dict_dir = self.app_args.dict_dir.clone();
        PluginWorker::<DictPlugin, DictPluginResult>::launch(
            &sidebar_sender,
            move || DictPlugin::new(dict_dir.as_str()).unwrap(),
            &input_broadcast,
        );

        PluginWorker::<Calculator, CalcResult>::launch(
            &sidebar_sender,
            || Calculator::new(),
            &input_broadcast,
        );
    }

    pub fn new_window(&mut self) -> RGWindow {
        let window = RGWindow::new(
            &self.app,
            self.app_msg_sender.clone(),
            self.app_msg_receiver.clone(),
            self.input_sender.clone(),
            self.input_receiver.clone(),
            self.selection_change_sender.clone(),
            self.selection_change_receiver.clone(),
            self.sidebar_sender.clone(),
            self.sidebar_receiver.clone(),
        );
        self.window.replace(window.clone());
        return window
    }
}
