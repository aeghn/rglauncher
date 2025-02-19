mod application;
mod arguments;
mod constants;
mod iconcache;
mod inputbar;
mod launcher;
mod pluginpreview;
mod resulthandler;
mod sidebar;
mod sidebarrow;
mod window;

use chin_tools::AResult;
use clap::Parser;
use rglcore::config::Config;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tracing::*;

use gtk::prelude::*;

use crate::application::RGLApplication;
use crate::launcher::LauncherMsg;
use flume::Sender;
use std::os::unix::net::{UnixListener, UnixStream};

pub fn daemon() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_thread_ids(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let (launcher_tx, launcher_rx) = flume::unbounded();

    let app_msg_tx = launcher_tx.clone();
    std::thread::spawn(move || {
        build_uds(&app_msg_tx).expect("unable to build unix domain socket");
    });

    let mut app = RGLApplication::new();

    let arguments = arguments::Arguments::parse();
    let config = Arc::new(Config::read_from_toml_file(arguments.config_file.as_ref()));
    iconcache::set_icon_dirs(config.common.icon_paths.clone());

    let launcher = launcher::Launcher::new(app.clone(), config, &launcher_tx, &launcher_rx);

    app.set_launcher(launcher);
    app.set_hold();

    let empty_args: Vec<String> = vec![];
    app.run_with_args(&empty_args);
}

fn build_uds(app_msg_tx: &Sender<LauncherMsg>) -> AResult<()> {
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
                    info!("Creating new window.");
                    app_msg_tx.send(LauncherMsg::NewWindow)?;
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

pub fn try_communicate() -> AResult<bool> {
    match UnixStream::connect(constants::UNIX_SOCKET_PATH) {
        Ok(mut stream) => {
            stream.write_all("new_window".as_bytes())?;
            Ok(true)
        }
        Err(_) => {
            daemon();
            Ok(true)
        }
    }
}

fn main() {
    try_communicate().expect("unable to communicate");
}
