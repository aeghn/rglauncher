use std::borrow::Borrow;



use glib::{clone, MainContext, PRIORITY_DEFAULT_IDLE};
use gtk::{self, Entry, ScrolledWindow, traits::{WidgetExt, GtkWindowExt, BoxExt}};
use gio::prelude::*;
use gtk::prelude::*;
use gtk::gdk;
use gtk::Inhibit;
use glib::{BoxedAnyObject};

use gtk::PolicyType::Never;


use crate::{plugin_worker, plugins::{PluginResult}};



use crate::inputbar::InputMessage;
use crate::plugin_worker::PluginMessage;
use crate::plugins::clipboard::{ClipboardPlugin};

use crate::sidebar::{SidebarMsg};

pub struct Launcher {
    input_bar: Entry,
    preview: ScrolledWindow
}

impl Launcher {
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        Launcher::build_window(window)
    }

    pub fn build_window(window: &gtk::ApplicationWindow) -> Self {
        let (input_tx, input_rx) = flume::unbounded::<InputMessage>();
        let (plugin_tx, plugin_rx) = flume::unbounded::<PluginMessage>();
        let (_result_sender, _result_receiver) = flume::unbounded::<Vec<Box<dyn PluginResult>>>();

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        window.set_child(Some(&main_box));

        let input_bar = crate::inputbar::get_input_bar(input_tx.clone());
        main_box.append(&input_bar);

        let bottom_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .vexpand(true)
            .build();
        main_box.append(&bottom_box);

        let mut sidebar = crate::sidebar::Sidebar::new();
        let sidebar_window = &sidebar.scrolled_window;
        bottom_box.append(sidebar_window);

        let preview_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(Never)
            .build();
        bottom_box.append(&preview_window);

        Launcher::setup_keybindings(&window, &sidebar.selection_model,
                               &sidebar.list_view, &input_bar);

        {
            let plugin_tx = plugin_tx.clone();
            let sidebar_receiver = sidebar.plugin_result_sender.clone();
            MainContext::ref_thread_default().spawn_local(async move {
                loop {
                    if let Ok(input_message) = input_rx.recv_async().await {
                        match input_message {
                            InputMessage::TextChange(text) => {
                                sidebar_receiver.send(SidebarMsg::TextChanged(text.clone())).unwrap();
                                plugin_tx.send(PluginMessage::Input(text)).unwrap();
                            }
                            InputMessage::EmitEnter => {}
                        }
                    }
                }
            });
        }


        let clipboard = ClipboardPlugin::new(crate::constant::STORE_DB);
        plugin_worker::PluginWorker::<ClipboardPlugin>::launch(&sidebar.plugin_result_sender,
                                                               clipboard,
                                                               &plugin_rx);

        {
            MainContext::ref_thread_default().spawn_local_with_priority(
                PRIORITY_DEFAULT_IDLE,
                async move {
                    sidebar.receive_msgs().await;
                });
        }

        Launcher {
            input_bar,
            preview: preview_window,
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
