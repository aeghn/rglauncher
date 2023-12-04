mod imp;

use crate::icon_cache;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

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

        imp.image
            .set_from_gicon(icon_cache::get_icon(plugin_result.sidebar_icon_name().as_str()).get());

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
    }
}
