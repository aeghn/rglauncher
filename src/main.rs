// #![no_main]

mod launcher;
mod inputbar;
mod shared;
mod sidebar;
mod plugins;
mod sidebar_row;
mod constant;
mod util;
mod dispatcher;
pub mod plugin_worker;


use tracing::*;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_tree::HierarchicalLayer;
use gtk::gdk::*;
use gtk::prelude::*;
use gtk::*;

const APP_ID: &str = "org.codeberg.wangzh.rglauncher";

// #[no_mangle]
// #[tokio::main]
fn main() {
    tracing_subscriber::registry()
        .with(HierarchicalLayer::new(2))
        .with(EnvFilter::from_default_env())
        .init();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| load_css());

    app.connect_activate(activate);

    info!("Ready.");
    app.run();
}

fn load_css() {
    info!("begin to load css info.");
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../resources/style.css"));

    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
    );

    info!("finished loading css info.");
}

fn activate(app: &Application) {
    info!("Activate.");
    let window = gtk::ApplicationWindow::builder()
        .default_width(800)
        .default_height(600)
        .application(app)
        .resizable(false)
        .title("Launcher")
        .build();

    let launcher = launcher::Launcher::new(&window);

    let settings = gtk::Settings::default().unwrap();
    settings.set_gtk_icon_theme_name(Some(&"ePapirus"));

    info!("Window show.");
    window.show();
}
