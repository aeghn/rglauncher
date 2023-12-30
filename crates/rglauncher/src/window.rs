use crate::arguments::Arguments;
use crate::icon_cache;
use crate::inputbar::{InputBar, InputMessage};
use crate::resulthandler::ResultHolder;
use crate::sidebar::SidebarMsg;
use backend::plugindispatcher::DispatchMsg;
use backend::ResultMsg;
use flume::Sender;
use glib::{clone, MainContext};
use gtk::prelude::EditableExt;
use gtk::prelude::EntryBufferExtManual;
use gtk::traits::{BoxExt, EntryExt, GtkWindowExt, WidgetExt};
use gtk::{gdk, Application, ApplicationWindow, Orientation};
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::SeqCst;
use tracing::info;
use crate::launcher::LauncherMsg;
use crate::pluginpreview::Preview;

#[derive(Clone)]
pub struct RGWindow {
    id: i32,

    window: ApplicationWindow,
    input_bar: InputBar,
    preview: Preview,

    dispatch_sender: Sender<DispatchMsg>,

    sidebar_sender: Sender<SidebarMsg>,

    result_sender: Sender<ResultMsg>,
}

static WINDOW_ID_COUNT: AtomicI32 = AtomicI32::new(0);

impl RGWindow {
    pub fn new(app: &Application,
               arguments: Arc<Arguments>,
               dispatch_sender: &Sender<DispatchMsg>,
               launcher_sender: &Sender<LauncherMsg>) -> Self {
        let id = WINDOW_ID_COUNT.fetch_add(1, SeqCst);

        let (sidebar_sender, sidebar_receiver) = flume::unbounded();
        let (preview_sender, preview_receiver) = flume::unbounded();

        let result_sender = ResultHolder::start(launcher_sender, dispatch_sender, &sidebar_sender, &preview_sender);

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
        info!("Show window");
        window.show();

        let input_bar = InputBar::new(&result_sender, id);
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

        // bottom_box.append(&gtk::Separator::builder().orientation(Orientation::Vertical).vexpand(true).build());

        let preview = Preview::new(preview_sender, preview_receiver);
        bottom_box.append(&preview.preview_window.clone());

        preview.loop_recv(&arguments);

        Self {
            id,
            window,
            input_bar,
            preview,

            dispatch_sender: dispatch_sender.clone(),
            result_sender: result_sender.clone(),

            sidebar_sender,
        }
    }

    fn setup_keybindings(&self) {
        let controller = gtk::EventControllerKey::new();
        let sidebar_sender = self.sidebar_sender.clone();
        let result_sender = self.result_sender.clone();
        let entry = self.input_bar.entry.clone();
        let inputbar_sender = self.input_bar.input_sender.clone();
        let window = &self.window;
        let rg_window = self.clone();

        controller.connect_key_pressed(clone!(@strong window,
            @strong entry => move |_, key, _keycode, _| {
            match key {
                gdk::Key::Up => {
                        sidebar_sender.send(SidebarMsg::PreviousItem).unwrap();
                        inputbar_sender.send(InputMessage::Focus).expect("unable to send focus");
                        glib::Propagation::Stop
                    }
                gdk::Key::Down => {
                        sidebar_sender.send(SidebarMsg::NextItem).unwrap();
                        inputbar_sender.send(InputMessage::Focus).expect("unable to send focus");
                        glib::Propagation::Stop
                    }
                gdk::Key::Escape => {
                        rg_window.hide_window();
                        glib::Propagation::Stop
                    }
                gdk::Key::Return => {
                        result_sender.send(ResultMsg::SelectSomething).expect("select something");
                        inputbar_sender.send(InputMessage::Clear).expect("unable to clear");
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
    }

    pub fn show_window(&self) {
        self.window.show();
    }

    pub fn hide_window(&self) {
        let inputbar_sender = self.input_bar.input_sender.clone();
        let window = self.window.clone();

        glib::idle_add_local_once(move || {
            window.destroy();
            inputbar_sender.send(InputMessage::Clear).expect("unable to clear");
        });
    }
}
