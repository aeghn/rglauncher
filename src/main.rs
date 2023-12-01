// #![no_main]

mod arguments;
mod constant;
mod icon_cache;
mod inputbar;
mod launcher;
pub mod plugin_worker;
mod plugins;
mod preview;
mod user_input;
mod sidebar;
mod sidebar_row;
mod util;

use clap::Parser;
use tracing::*;

use gtk::gdk::*;
use gtk::prelude::*;
use gtk::*;

const APP_ID: &str = "org.codeberg.wangzh.rglauncher";

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();


    let arguments = arguments::Arguments::parse();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());

    app.connect_activate(move |app| {
        activate(app, &arguments)
    });

    let empty: Vec<String> = vec![];
    app.run_with_args(&empty);
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

fn activate(app: &Application, args: &arguments::Arguments) {
    let window = gtk::ApplicationWindow::builder()
        .default_width(800)
        .default_height(600)
        .application(app)
        .resizable(false)
        .title("Launcher")
        .decorated(false)
        .build();
    let launcher = launcher::Launcher::new(&window);
    // let settings = Settings::default().unwrap();
    // settings.set_gtk_icon_theme_name(Some(&"ePapirus"));

    window.show();
    launcher.post_actions(args);
}
