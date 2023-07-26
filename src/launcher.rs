use std::borrow::Borrow;
use std::process::exit;
use async_broadcast::{Receiver, Sender};

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

        let mut sidebar = crate::sidebar::Sidebar::new(input_bar.input_broadcast.clone());
        let sidebar_window = &sidebar.scrolled_window;
        bottom_box.append(sidebar_window);

        let mut sidebar_worker = sidebar.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            sidebar_worker.loop_recv().await;
        });

        // let mut sidebar_worker = sidebar.clone();
        // MainContext::ref_thread_default().spawn_local(async move {
        //     sidebar_worker.loop_recv_input().await;
        // });

        let preview = Preview::new(selection_change_receiver.clone());
        bottom_box.append(&preview.preview_window.clone());

        let preview_worker = preview.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            preview_worker.loop_recv().await;
        });

        // Launcher::setup_keybindings(&window, &sidebar.selection_model,
        //                        &sidebar.list_view, &input_bar.entry);

        // {
        //     let plugin_tx = plugin_tx.clone();
        //     let sidebar_receiver = sidebar.plugin_result_sender.clone();
        //     MainContext::ref_thread_default().spawn_local(async move {
        //         loop {
        //             if let Ok(input_message) = input_rx.recv_async().await {
        //                 match input_message {
        //                     InputMessage::TextChange(text) => {
        //                         sidebar_receiver.send(SidebarMsg::TextChanged(text.clone())).unwrap();
        //                         plugin_tx.send(PluginMessage::Input(text)).unwrap();
        //                     }
        //                     InputMessage::EmitEnter => {}
        //                 }
        //             }
        //         }
        //     });
        // }

        let clipboard = ClipboardPlugin::new(crate::constant::STORE_DB);
        plugin_worker::PluginWorker::<ClipboardPlugin, ClipPluginResult>::launch(&sidebar.sidebar_sender,
                                                               clipboard,
                                                               &input_bar.input_broadcast);

        let plugin = AppPlugin::new();
        plugin_worker::PluginWorker::<AppPlugin, AppResult>::launch(&sidebar.sidebar_sender,
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
                         selection_model: &gtk::SingleSelection,
                         list_view: &gtk::ListView,
                         entry: &Entry) {
        let controller = gtk::EventControllerKey::new();

        controller.connect_key_pressed(clone!(@strong window,
            @strong selection_model,
            @strong entry,
            @strong list_view => move |_, key, _keycode, _| {

                match key {
                    gdk::Key::Up => {
                        let new_selection = if selection_model.selected() > 0 {
                            selection_model.selected() - 1
                        } else {
                            0
                        };
                        selection_model.select_item(new_selection, true);
                        list_view.activate_action("list.scroll-to-item", Some(&new_selection.to_variant())).unwrap();

                        Inhibit(false)
                    }
                    gdk::Key::Down => {
                        let new_selection = if selection_model.n_items() > 0 {
                            std::cmp::min(selection_model.n_items() - 1, selection_model.selected() + 1)
                        } else {
                            0
                        };
                        selection_model.select_item(new_selection, true);
                        list_view.activate_action("list.scroll-to-item", Some(&new_selection.to_variant())).unwrap();

                        Inhibit(false)
                    }
                    gdk::Key::Escape => {
                        window.destroy();
                        Inhibit(false)
                    }
                    gdk::Key::Return => {
                        let item = selection_model.selected_item();
                        if let Some(boxed) = item {
                            let tt = boxed.downcast_ref::<BoxedAnyObject>().unwrap().borrow::<Box<dyn PluginResult>>();
                            tt.on_enter();
                            window.destroy();
                        }

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
