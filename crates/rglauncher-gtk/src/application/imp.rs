use crate::launcher::Launcher;
use gio::ApplicationHoldGuard;
use gtk::gdk::Display;
use gtk::subclass::prelude::*;
use gtk::{glib, style_context_add_provider_for_display, CssProvider, Settings};
use std::cell::OnceCell;
use tracing::info;

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
                let config = launcher.config.as_ref();

                let settings = Settings::default().expect("Failed to create GTK settings.");
                match config.ui.as_ref() {
                    Some(config) => {
                        settings.set_gtk_icon_theme_name(Some(config.icon_theme.as_str()));
                    }
                    None => {}
                }

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
        provider.load_from_data(include_str!("../style.css"));

        style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
