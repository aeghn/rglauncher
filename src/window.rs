use crate::inputbar::{InputBar, InputMessage};
use crate::launcher::AppMsg;
use crate::preview::Preview;
use crate::sidebar::SidebarMsg;
use flume::{Receiver, Sender};
use gdk::Key;
use gio::traits::ApplicationExt;
use glib::{clone, BoxedAnyObject, GStr, MainContext};
use gtk::prelude::EditableExt;
use gtk::prelude::EntryBufferExtManual;
use gtk::traits::{BoxExt, EntryExt, GtkWindowExt, WidgetExt};
use gtk::{gdk, Application, ApplicationWindow};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone)]
pub struct RGWindow {
    window: ApplicationWindow,
    input_bar: InputBar,
    preview: Preview,
    sidebar_sender: Sender<SidebarMsg>,
    selection_change_receiver: flume::Receiver<BoxedAnyObject>,
}

impl RGWindow {
    pub fn new(
        app: &Application,
        app_msg_sender: Sender<AppMsg>,
        input_sender: async_broadcast::Sender<Arc<InputMessage>>,
        input_receiver: async_broadcast::Receiver<Arc<InputMessage>>,
        selection_change_sender: Sender<BoxedAnyObject>,
        selection_change_receiver: flume::Receiver<BoxedAnyObject>,
        sidebar_sender: Sender<SidebarMsg>,
        sidebar_receiver: Receiver<SidebarMsg>,
    ) -> Self {
        let window = ApplicationWindow::builder()
            .default_width(800)
            .default_height(600)
            .application(app)
            .resizable(false)
            .title("RGLauncher")
            .decorated(false)
            .build();

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        window.set_child(Some(&main_box));

        let input_bar = InputBar::new(&input_sender);
        main_box.append(&input_bar.entry);

        let bottom_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .vexpand(true)
            .build();
        main_box.append(&bottom_box);

        let sidebar = crate::sidebar::Sidebar::new(
            input_receiver.clone(),
            sidebar_receiver.clone(),
            selection_change_sender.clone(),
            app_msg_sender.clone(),
        );
        let sidebar_window = &sidebar.scrolled_window;
        bottom_box.append(sidebar_window);

        let mut sidebar_worker = sidebar.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            sidebar_worker.loop_recv().await;
        });

        let preview = Preview::new();
        bottom_box.append(&preview.preview_window.clone());

        Self {
            window,
            input_bar,
            preview,
            sidebar_sender,
            selection_change_receiver,
        }
    }

    fn setup_keybindings(&self) {
        let controller = gtk::EventControllerKey::new();
        let sender = self.sidebar_sender.clone();
        let entry = self.input_bar.entry.clone();
        let window = &self.window;

        controller.connect_key_pressed(clone!(@strong window,
            @strong entry => move |_, key, _keycode, _| {
            match key {
                gdk::Key::Up => {
                        sender.send(SidebarMsg::PreviousItem).unwrap();
                        glib::Propagation::Proceed
                    }
                gdk::Key::Down => {
                        sender.send(SidebarMsg::NextItem).unwrap();
                        glib::Propagation::Proceed
                }
                gdk::Key::Escape => {
                        window.hide();
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

    pub fn prepare(&self) {
        self.setup_keybindings();
        let selection_change_receiver = self.selection_change_receiver.clone();
        let preview = self.preview.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            preview.loop_recv(selection_change_receiver).await;
        });
    }

    pub fn show_window(&self) {
        self.window.show();
    }

    pub fn hide_window(&self) {
        let entry = self.input_bar.entry.clone();
        let window = self.window.clone();

        glib::idle_add_local_once(move || {
            window.hide();
            entry.set_text(&"".to_string());
        });
    }
}
