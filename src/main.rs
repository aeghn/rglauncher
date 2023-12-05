mod arguments;
mod constants;
mod icon_cache;
mod inputbar;
mod launcher;
mod plugins;
mod pluginworker;
mod preview;
mod sidebar;
mod sidebarrow;
mod userinput;
mod util;
mod window;

use clap::Parser;
use gio::File;
use gio::FileType::Directory;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::*;

use gtk::gdk::*;
use gtk::prelude::*;
use gtk::*;

use crate::launcher::{AppMsg, Launcher};
use crate::window::RGWindow;
use flume::{Receiver, Sender};
use fragile::Fragile;
use glib::{MainContext, MainLoop};
use std::os::unix::net::{UnixListener, UnixStream};

const APP_ID: &str = "org.codeberg.wangzh.rglauncher";

fn main() {
    try_communicate().expect("unable to create socket");
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../resources/style.css"));

    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn activate(app: &Application, app_msg_sender: Sender<AppMsg>, app_msg_receiver: Receiver<AppMsg>) {
    let arguments = arguments::Arguments::parse();

    let launcher =
        launcher::Launcher::new(app.clone(), arguments, app_msg_sender, app_msg_receiver);

    launcher.launch_plugins();

    let window = launcher.new_window();

    window.prepare();
    window.show_window();
}

fn start_new() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let (app_msg_sender, app_msg_receiver) = flume::unbounded();

    {
        let app_msg_sender = app_msg_sender.clone();
        std::thread::spawn(move || {
            build_uds(&app_msg_sender).expect("unable to build unix domain socket");
        });
    }

    gtk::init();

    let main_loop = glib::MainLoop::new(None, false);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());

    {
        let app_msg_sender = app_msg_sender.clone();
        let app_msg_receiver = app_msg_receiver.clone();
        app.connect_activate(move |app| {
            activate(app, app_msg_sender.clone(), app_msg_receiver.clone());
        });
    }

    let empty: Vec<String> = vec![];
    let _ = app.hold();
    app.run_with_args(&empty);

    main_loop.run();
}

fn build_uds(app_msg_sender: &Sender<AppMsg>) -> anyhow::Result<()> {
    if !Path::new(constants::TMP_DIR).exists() {
        std::fs::create_dir(constants::TMP_DIR)?;
    }

    if Path::new(constants::UNIX_SOCKET_PATH).exists() {
        std::fs::remove_file(constants::UNIX_SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(constants::UNIX_SOCKET_PATH)?;
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut response = String::new();
                stream.read_to_string(&mut response)?;
                info!("Got Echo {}", response);

                if response == "new_window" {
                    app_msg_sender.send(AppMsg::NewWindow)?;
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn try_communicate() -> anyhow::Result<bool> {
    match UnixStream::connect(constants::UNIX_SOCKET_PATH) {
        Ok(mut stream) => {
            stream.write_all("new_window".as_bytes())?;
            Ok(true)
        }
        Err(_) => {
            start_new();
            Ok(true)
        }
    }
}
