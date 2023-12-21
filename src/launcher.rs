use flume::{Receiver, Sender};
use glib::{clone, BoxedAnyObject, MainContext};
use std::sync::{Arc, RwLock};
use std::thread::sleep;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::ResponseType::No;
use gtk::{gdk, Application};

use crate::arguments;
use crate::arguments::Arguments;
use tracing::{error, info};

use crate::inputbar::{InputBar, InputMessage};


use crate::sidebar::SidebarMsg;
use crate::window::RGWindow;

#[derive(Clone)]
pub struct Launcher {
    app: Application,
    app_args: Arguments,

    app_msg_sender: Sender<AppMsg>,
    app_msg_receiver: Receiver<AppMsg>,
    selection_change_sender: Sender<BoxedAnyObject>,
    selection_change_receiver: Receiver<BoxedAnyObject>,
    sidebar_sender: Sender<SidebarMsg>,
    sidebar_receiver: Receiver<SidebarMsg>,

    db: Arc<RwLock<Option<rusqlite::Connection>>>,
}

pub enum AppMsg {
    SelectSomething,
    Exit,
    NewWindow,
}

impl Launcher {
    pub fn new(
        application: Application,
        arguments: Arguments,
        app_msg_sender: Sender<AppMsg>,
        app_msg_receiver: Receiver<AppMsg>,
    ) -> Self {
        let (selection_change_sender, selection_change_receiver) = flume::unbounded();
        let (sidebar_sender, sidebar_receiver) = flume::unbounded();

        Launcher {
            app: application,
            app_args: arguments,

            app_msg_sender,
            app_msg_receiver,
            selection_change_sender,
            selection_change_receiver,
            sidebar_sender,
            sidebar_receiver,

            db: Arc::new(RwLock::default()),
        }
    }

    pub fn launch_plugins(&self) {
        let sidebar_sender = self.sidebar_sender.clone();
    }

    pub fn message_handler(&self) {
        let oself = self.clone();
    }

/*     pub fn new_window(&self) -> RGWindow {
/*         let app_msg_receiver = self.app_msg_receiver.clone();
 *//*         let window = RGWindow::new(
            &self.app,
            self.app_msg_sender.clone(),
            self.selection_change_sender.clone(),
            self.selection_change_receiver.clone(),
            self.sidebar_sender.clone(),
            self.sidebar_receiver.clone(),
        );

        let win = window.clone();
/*         let input_sender = self.input_sender.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            loop {
                match app_msg_receiver.recv_async().await {
                    Ok(msg) => match msg {
                        AppMsg::Exit => {}
                        AppMsg::NewWindow => {
                            let _ = input_sender.broadcast(Arc::new(InputMessage::RefreshContent));
                            win.show_window();
                        }
                        AppMsg::SelectSomething => {
                            win.hide_window();
                        }
                    },
                    Err(_) => {}
                }
            }
        }); */

        return window; */

    } */
}
