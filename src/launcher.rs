use std::borrow::Borrow;
use std::process::exit;

use glib::{clone, MainContext, PRIORITY_DEFAULT_IDLE};
use glib::{BoxedAnyObject};
use gio::prelude::*;
use gtk::{self, Entry, ScrolledWindow, traits::{WidgetExt, GtkWindowExt, BoxExt}};
use gtk::prelude::*;
use gtk::gdk;
use gtk::Inhibit;
use gtk::PolicyType::Never;
use tracing::error;

use crate::{plugin_worker, plugins::{PluginResult}};
use crate::inputbar::{InputBar};
use crate::plugin_worker::PluginWorker;
use crate::plugins::app::{AppPlugin, AppResult};
use crate::plugins::clipboard::{ClipboardPlugin, ClipPluginResult};
use crate::preview::Preview;

use crate::sidebar::{SidebarMsg};

pub struct Launcher {
    input_bar: InputBar,
    preview: Preview,

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

        let mut sidebar = crate::sidebar::Sidebar::new(
            input_bar.input_broadcast.clone(),
            sidebar_receiver.clone(),
            selection_change_sender.clone());
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
            preview_worker.loop_recv(selection_change_receiver.clone()).await;
        });

        Launcher::setup_keybindings(&window, sidebar_sender.clone(),
                                    &input_bar.entry);

        let clipboard = ClipboardPlugin::new(crate::constant::STORE_DB);
        PluginWorker::<ClipboardPlugin, ClipPluginResult>::launch(&sidebar_sender,
                                                               clipboard,
                                                               &input_bar.input_broadcast);

        let plugin = AppPlugin::new();
        PluginWorker::<AppPlugin, AppResult>::launch(&sidebar_sender,
                                                     plugin,
                                                     &input_bar.input_broadcast);

        window.connect_destroy(|_| {
            exit(0);
        });

        Launcher {
            input_bar,
            preview,
        }
    }

    fn setup_keybindings(window: &gtk::ApplicationWindow,
                         sidebar_sender: flume::Sender<SidebarMsg>,
                         entry: &Entry) {
        let controller = gtk::EventControllerKey::new();

        controller.connect_key_pressed(clone!(@strong window,
            @strong entry=> move |_, key, _keycode, _| {
            match key {
                gdk::Key::Up => {
                    sidebar_sender.send(SidebarMsg::PreviousItem);
                    Inhibit(false)
                }
                gdk::Key::Down => {
                    sidebar_sender.send(SidebarMsg::NextItem);
                    Inhibit(false)
                }
                gdk::Key::Escape => {
                    window.destroy();
                    Inhibit(false)
                }
                gdk::Key::Return => {
                    sidebar_sender.send(SidebarMsg::Enter);
                    Inhibit(false)
                }
                _ => {
                    if !(key.is_lower() && key.is_upper()) {
                        if let Some(key_name) = key.name() {
                            let buffer = entry.buffer();

                            let content = buffer.text();
                            buffer.insert_text((content.len()) as u16, key_name);
                        }
                    }

                    Inhibit(false)
                }
            }
        }));
        window.add_controller(controller);
    }
}
