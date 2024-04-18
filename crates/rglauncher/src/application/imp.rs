use crate::launcher::Launcher;
use gtk::gdk::Display;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, style_context_add_provider_for_display, CssProvider, Settings};
use std::cell::OnceCell;
use tracing::info;
use gio::ApplicationHoldGuard;

use std::borrow::Borrow;
#[derive(Default)]
pub struct RGLApplication {
    pub(crate) launcher: OnceCell<Launcher>,
    pub background_hold: OnceCell<ApplicationHoldGuard>,
}

#[glib::object_subclass]
impl ObjectSubclass for RGLApplication {
    const NAME: &'static str = "ExApplication";
    type Type = super::RGLApplication;
    type ParentType = gtk::Application;
}

impl ObjectImpl for RGLApplication {}
impl ApplicationImpl for RGLApplication {
    fn activate(&self) {
        info!("Activating");
        self.parent_activate();
        match self.launcher.borrow().get() {
            None => {}
            Some(launcher) => {
                let arguments = launcher.app_args.as_ref();

                let settings = Settings::default().expect("Failed to create GTK settings.");
                settings.set_gtk_icon_theme_name(Some(arguments.icon.as_str()));

                launcher.new_window();
            }
        }
    }

    fn startup(&self) {
        self.parent_startup();
        self.load_css();
    }
}
impl GtkApplicationImpl for RGLApplication {}

impl RGLApplication {
    fn load_css(&self) {
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("../../../../res/style.css"));

        style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
