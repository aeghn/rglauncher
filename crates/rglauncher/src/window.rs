use crate::application::RGLApplication;
use crate::arguments::Arguments;
use crate::icon_cache;
use crate::inputbar::{InputBar, InputMessage};
use crate::launcher::LauncherMsg;
use crate::pluginpreview::Preview;
use crate::resulthandler::ResultHolder;
use crate::sidebar::SidebarMsg;
use backend::plugindispatcher::DispatchMsg;
use backend::ResultMsg;
use flume::{Receiver, RecvError, Sender};
use glib::{clone, MainContext};
use gtk::prelude::EditableExt;
use gtk::prelude::EntryBufferExtManual;
use gtk::traits::{BoxExt, EntryExt, GtkWindowExt, WidgetExt};
use gtk::{gdk, Application, ApplicationWindow, Orientation};
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use tracing::info;
use backend::userinput::UserInput;

pub enum WindowMsg {
    Close,
}

#[derive(Clone)]
pub struct RGWindow {
    window: ApplicationWindow,
    input_bar: InputBar,
    preview: Preview,

    pub window_sender: Sender<WindowMsg>,
    window_receiver: Receiver<WindowMsg>,

    dispatch_sender: Sender<DispatchMsg>,
    sidebar_sender: Sender<SidebarMsg>,
    result_sender: Sender<ResultMsg>,
}

impl RGWindow {
    pub fn new(
        app: &RGLApplication,
        arguments: Arc<Arguments>,
        dispatch_sender: &Sender<DispatchMsg>,
        launcher_sender: &Sender<LauncherMsg>,
    ) -> Self {
        let (sidebar_sender, sidebar_receiver) = flume::unbounded();
        let (preview_sender, preview_receiver) = flume::unbounded();
        let (window_sender, window_receiver) = flume::unbounded();

        let result_sender = ResultHolder::start(
            launcher_sender,
            dispatch_sender,
            &sidebar_sender,
            &preview_sender,
        );

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
            .css_classes(["main-box"])
            .build();

        window.set_child(Some(&main_box));

        let input_bar = InputBar::new(&result_sender, &window_sender);
        main_box.append(&input_bar.entry);

        let bottom_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .vexpand(true)
            .build();
        main_box.append(&bottom_box);

        let mut sidebar = crate::sidebar::Sidebar::new(
            &result_sender,
            sidebar_sender.clone(),
            sidebar_receiver.clone(),
        );
        let sidebar_window = &sidebar.scrolled_window;
        let sidebar_sender = sidebar.sidebar_sender.clone();
        bottom_box.append(sidebar_window);

        sidebar.loop_recv();

        let preview = Preview::new(preview_sender, preview_receiver);
        bottom_box.append(&preview.preview_window.clone());

        preview.loop_recv(&arguments);

        window.present();

        result_sender.send(ResultMsg::UserInput(Arc::new(UserInput::new("")))).expect("unable to submit initial input");

        window.connect_destroy(|win| {
            info!("window destroied");
        });

        Self {
            window,
            input_bar,
            preview,

            window_sender,
            window_receiver,

            dispatch_sender: dispatch_sender.clone(),
            result_sender: result_sender.clone(),

            sidebar_sender,
        }
    }

    fn receive_messages(&self) {
        let window = self.window.clone();
        let window_receiver = self.window_receiver.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            loop {
                match window_receiver.recv_async().await {
                    Ok(WindowMsg::Close) => {
                        RGWindow::close_window(&window.clone());
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    fn setup_keybindings(&self) {
        let controller = gtk::EventControllerKey::new();
        let sidebar_sender = self.sidebar_sender.clone();
        let result_sender = self.result_sender.clone();
        let entry = self.input_bar.entry.clone();
        let inputbar_sender = self.input_bar.input_sender.clone();
        let window_sender = self.window_sender.clone();
        let window = &self.window;
        let rg_window = self.clone();

        controller.connect_key_pressed(clone!(@strong window,
        @strong entry => move |_, key, _keycode, _| {
        match key {
            gdk::Key::Up => {
                    sidebar_sender.send(SidebarMsg::PreviousItem).unwrap();
                    glib::Propagation::Stop
                }
            gdk::Key::Down => {
                    sidebar_sender.send(SidebarMsg::NextItem).unwrap();
                    glib::Propagation::Stop
                }
            gdk::Key::Escape => {
                        window_sender.send(WindowMsg::Close).expect("unable to close window");
                    glib::Propagation::Stop
                }
            gdk::Key::Return => {
                    result_sender.send(ResultMsg::SelectSomething).expect("select something");
                    inputbar_sender.send(InputMessage::Clear).expect("unable to clear");
                    window_sender.send(WindowMsg::Close).expect("unable to close window");
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

    pub fn setup_one(
        app: &RGLApplication,
        arguments: Arc<Arguments>,
        dispatch_sender: &Sender<DispatchMsg>,
        launcher_sender: &Sender<LauncherMsg>,
    ) {
        let window = Self::new(app, arguments, dispatch_sender, launcher_sender);

        window.setup_keybindings();
        window.receive_messages();
    }

    pub fn show_window(&self) {
        self.window.show();
    }

    pub fn close_window(window: &ApplicationWindow) {
        let window = window.clone();

        glib::idle_add_local_once(move || {
            window.destroy();
        });
    }
}
