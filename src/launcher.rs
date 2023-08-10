use std::sync::{Arc, RwLock};
use glib::{clone, MainContext};

use gio::prelude::*;
use gtk::gdk;
use gtk::prelude::*;
use gtk::{
    self,
    traits::{BoxExt, GtkWindowExt, WidgetExt},
    ApplicationWindow, Entry,
};

use tracing::error;

use crate::inputbar::InputBar;
use crate::plugin_worker::PluginWorker;
use crate::plugins::app::{AppPlugin, AppResult};
use crate::plugins::clipboard::{ClipPluginResult, ClipboardPlugin};
use crate::plugins::mdict::{MDictPlugin, MDictPluginResult};
use crate::plugins::windows::{HyprWindowResult, HyprWindows};
use crate::preview::Preview;

use crate::sidebar::SidebarMsg;

pub struct Launcher {
    input_bar: InputBar,
    preview: Preview,
    window: gtk::ApplicationWindow,
    db: Arc<RwLock<Option<rusqlite::Connection>>>
}

pub enum AppMsg {
    Exit,
}

impl Launcher {
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        Launcher::build_window(window)
    }

    pub fn build_window(window: &gtk::ApplicationWindow) -> Self {
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        window.set_child(Some(&main_box));

        let input_bar = InputBar::new();
        main_box.append(&input_bar.entry);

        let bottom_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .vexpand(true)
            .build();
        main_box.append(&bottom_box);
        let (selection_change_sender, selection_change_receiver) = flume::unbounded();
        let (sidebar_sender, sidebar_receiver) = flume::unbounded();
        let (app_msg_sender, app_msg_receiver) = flume::unbounded();

        let sidebar = crate::sidebar::Sidebar::new(
            input_bar.input_broadcast.clone(),
            sidebar_receiver.clone(),
            selection_change_sender.clone(),
            app_msg_sender,
        );
        let sidebar_window = &sidebar.scrolled_window;
        bottom_box.append(sidebar_window);

        let mut sidebar_worker = sidebar.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            sidebar_worker.loop_recv().await;
        });

        let preview = Preview::new();
        bottom_box.append(&preview.preview_window.clone());

        let preview_worker = preview.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            preview_worker
                .loop_recv(selection_change_receiver.clone())
                .await;
        });

        Launcher::setup_keybindings(&window, sidebar_sender.clone(), &input_bar.entry);

        {
            let window = window.clone();
            MainContext::ref_thread_default().spawn_local(async move {
                Launcher::handle_app_msgs(app_msg_receiver, window).await;
            });
        }

        let clipboard = ClipboardPlugin::new(crate::constant::STORE_DB);
        PluginWorker::<ClipboardPlugin, ClipPluginResult>::launch(
            &sidebar_sender,
            clipboard,
            &input_bar.input_broadcast,
        );

        let plugin = AppPlugin::new();
        PluginWorker::<AppPlugin, AppResult>::launch(
            &sidebar_sender,
            plugin,
            &input_bar.input_broadcast,
        );

        PluginWorker::<HyprWindows, HyprWindowResult>::launch(
            &sidebar_sender,
            HyprWindows::new(),
            &input_bar.input_broadcast,
        );

        let mdict = MDictPlugin::new("/home/chin/.cache/rglauncher/mdict.db", vec![]);
        PluginWorker::<MDictPlugin, MDictPluginResult>::launch(
            &sidebar_sender,
            mdict,
            &input_bar.input_broadcast,
        );

        Launcher {
            window: window.clone(),
            input_bar,
            preview,
            db: Arc::new(RwLock::default()),
        }
    }

    async fn handle_app_msgs(app_msg_receiver: flume::Receiver<AppMsg>, window: ApplicationWindow) {
        loop {
            match app_msg_receiver.recv_async().await {
                Ok(msg) => match msg {
                    AppMsg::Exit => match window.application() {
                        None => {
                            error!("unable to get this application.");
                        }
                        Some(app) => {
                            app.quit();
                        }
                    },
                },
                Err(_) => {}
            }
        }
    }

    fn setup_keybindings(
        window: &gtk::ApplicationWindow,
        sidebar_sender: flume::Sender<SidebarMsg>,
        entry: &Entry,
    ) {
        let controller = gtk::EventControllerKey::new();

        controller.connect_key_pressed(clone!(@strong window,
            @strong entry=> move |_, key, _keycode, _| {
            match key {
                gdk::Key::Up => {
                    sidebar_sender.send(SidebarMsg::PreviousItem).unwrap();
                    glib::Propagation::Proceed
                }
                gdk::Key::Down => {
                    sidebar_sender.send(SidebarMsg::NextItem).unwrap();
                    glib::Propagation::Proceed
                }
                gdk::Key::Escape => {
                    window.destroy();
                    glib::Propagation::Proceed
                }
                gdk::Key::Return => {
                    sidebar_sender.send(SidebarMsg::Enter).unwrap();

                    glib::Propagation::Proceed
                }
                _ => {
                    if !(key.is_lower() && key.is_upper()) {
                        if let Some(key_name) = key.name() {
                            let buffer = entry.buffer();

                            let content = buffer.text();
                            buffer.insert_text((content.len()) as u16, key_name);
                        }
                    }

                    glib::Propagation::Proceed
                }
            }
        }));
        window.add_controller(controller);
    }
}
