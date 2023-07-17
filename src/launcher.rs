use std::borrow::Borrow;
use glib::{clone, MainContext};
use gtk::{self, Entry, ScrolledWindow, traits::{WidgetExt, GtkWindowExt, BoxExt}};
use gio::prelude::*;
use gtk::prelude::*;
use gtk::gdk;
use gtk::Inhibit;
use glib::{BoxedAnyObject};
use gtk::Align::Center;
use gtk::PolicyType::Never;
use gtk::ResponseType::No;
use crate::{dispatcher, plugins::{PluginResult}};

use tracing::{error};
use crate::dispatcher::Dispatcher;
use crate::shared::UserInput;
use crate::sidebar::Sidebar;

pub struct Launcher {
    input_bar: Entry,
    sidebar: Sidebar,
    preview: ScrolledWindow
}

impl Launcher {
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        let (input_tx, input_rx) = flume::unbounded();

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

        let sidebar = crate::sidebar::Sidebar::new();
        let sidebar_window = &sidebar.scrolled_window;
        bottom_box.append(sidebar_window);

        let preview_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(Never)
            .build();
        bottom_box.append(&preview_window);

        Launcher::setup_keybindings(&window, &sidebar.selection_model,
                               &sidebar.list_view, &input_bar);



        Launcher {
            input_bar,
            sidebar,
            preview: preview_window,
        }

        // {
        //     let list_store = sidebar.list_store;
        //     dispatcher_rx.attach(None, move |r| {
        //         list_store.remove_all();
        //         for x in r.1 {
        //             list_store.append(&BoxedAnyObject::new(x));
        //         };
        //
        //         Continue(true)
        //     });
        // }
        //
        // let dispatcher = Dispatcher::new( dispatcher_tx);
        // input_rx.attach(None, move|ui| {
        //     dispatcher.handle_messages(ui);
        //     Continue(true)
        // });
        //
        //     input_bar.connect_activate(clone!(@weak selection_model => move |_| {
        //     let row_data = &selection_model.selected_item();
        //     if let Some(boxed) = row_data {
        //         let pr = boxed.downcast_ref::<BoxedAnyObject>().unwrap()
        //         .borrow::<Box<dyn PluginResult>>();
        //         pr.on_enter();
        //         gtk::main_quit();
        //     }
        // }));
    }

    fn setup_keybindings(window: &gtk::ApplicationWindow,
                         selection_model: &gtk::SingleSelection,
                         list_view: &gtk::ListView,
                         entry: &Entry) {
        let controller = gtk::EventControllerKey::new();

        controller.connect_key_pressed(clone!(@strong window,
            @strong selection_model,
            @strong entry,
            @strong list_view => move |_, key, keycode, _| {

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
