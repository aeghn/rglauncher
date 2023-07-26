mod imp;

use futures::future::err;
use gio::Icon;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::pango::WrapMode;
use tracing::error;


use crate::plugins::PluginResult;

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::Widget, gtk::Box;
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

        let icon = plugin_result.sidebar_icon();

        match icon {
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
                imp.content.set_label(Some(e.as_str()));
                if let Some(label) = imp.content.label_widget().and_downcast::<gtk::Label>() {
                    label.set_wrap(true);
                    label.set_wrap_mode(WrapMode::WordChar);
                }
            }
        };

        match plugin_result.sidebar_content() {
            None => {}
            Some(e) => {
                e.set_hexpand(true);
                imp.content.set_child(Some(&e));
            }
        };
        error!("set_sidebar");
    }
}
