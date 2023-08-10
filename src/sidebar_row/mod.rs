mod imp;

use gio::Icon;

use gtk::pango::WrapMode;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, Widget};

use crate::plugins::PluginResult;

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::Widget, gtk::Grid;
}

impl Default for SidebarRow {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_sidebar(&self, plugin_result: &dyn PluginResult) {
        let imp = self.imp();

        match plugin_result.sidebar_icon() {
            None => {
                let x1 = Icon::for_string("missing");
                match x1 {
                    Ok(x) => {
                        imp.image.set_from_gicon(&x);
                    }
                    Err(_) => {}
                }
            }
            Some(x) => {
                imp.image.set_from_gicon(&x);
            }
        };

        match plugin_result.sidebar_label() {
            None => {}
            Some(e) => {
                imp.title.set_label(e.as_str());
            }
        };

        match plugin_result.sidebar_content() {
            None => {}
            Some(e) => {
                imp.content.set_label(e.as_str());
            }
        };
    }

    pub fn unbind_all(&self) {
        let imp = self.imp();
        imp.image.clear();
        imp.content.set_label(&"");
        imp.title.set_label(&"");
    }
}
