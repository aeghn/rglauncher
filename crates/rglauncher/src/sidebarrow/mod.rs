mod imp;

use crate::iconcache;
use backend::util::string_utils;
use gtk::glib;
use gtk::subclass::prelude::*;

use backend::plugins::PluginResult;

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

    pub fn arrange_sidebar(&self, plugin_result: &dyn PluginResult) {
        let imp = self.imp();

        imp.image.set_from_gicon(iconcache::get_icon(plugin_result.icon_name()).get());

        imp.title.set_label(plugin_result.name());

        plugin_result.extra().map(|desc| {
            let desc = if desc.len() > 300 {
                string_utils::truncate(desc, 300)
            } else {
                desc
            };

            imp.extra.set_label(desc)
        });
            
    }

    pub fn unbind_all(&self) {
        let imp = self.imp();
        imp.image.clear();
    }
}
