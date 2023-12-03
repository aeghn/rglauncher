// #![no_main]

mod arguments;
mod constants;
mod icon_cache;
mod inputbar;
mod launcher;
mod plugin_worker;
mod plugins;
mod preview;
mod user_input;
mod sidebar;
mod sidebar_row;
mod util;
mod window;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use clap::Parser;
use gio::File;
use gio::FileType::Directory;
use tracing::*;

use gtk::gdk::*;
use gtk::prelude::*;
use gtk::*;

use std::os::unix::net::{UnixListener, UnixStream};
use fragile::Fragile;
use glib::MainContext;
use crate::launcher::Launcher;
use crate::window::RGWindow;

const APP_ID: &str = "org.codeberg.wangzh.rglauncher";

fn main() {
    ensure_unix_socket().expect("unable to create socket");
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

fn activate(launcher: Launcher) {
    let window = launcher.clone().new_window();

    window.prepare();
    window.show_window();
}

fn ensure_unix_socket() -> anyhow::Result<bool>{
    match UnixStream::connect(constants::UNIX_SOCKET_PATH) {
        Ok(mut stream) => {
            stream.write_all("new_window".as_bytes())?;
            Ok(true)
        }
        Err(_) => {
            tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .with_timer(tracing_subscriber::fmt::time::time())
                .init();

            let app = Application::builder().application_id(APP_ID).build();

            let arguments = arguments::Arguments::parse();
            let launcher = launcher::Launcher::new(app.clone(), arguments);
            let launcher_wrapper = Fragile::new(launcher.clone());

            app.connect_startup(|_| load_css());

            {
                let launcher = launcher.clone();
                launcher.launch_plugins();
                app.connect_activate(move |_| {
                    activate(launcher.clone());
                });
            }

            let empty: Vec<String> = vec![];
            app.run_with_args(&empty);


            std::thread::spawn(|| {
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

                            if response == "new_window" {
                                let launcher_wrapper = launcher_wrapper.clone();
                                MainContext::ref_thread_default().invoke(move || {
                                    // activate(launcher_wrapper.get().clone());
                                    match launcher_wrapper.get().window.clone() {
                                        None => {}
                                        Some(win) => {
                                            win.show_window();
                                        }
                                    }
                                });
                            }
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
            });
            Ok(true)
        }
    }
}