mod imp;

use crate::iconcache;
use gtk::glib;
use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;
use rglcore::util::string_utils;

use rglcore::plugins::PluginResult;

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

        imp.image
            .set_from_gicon(iconcache::get_icon(plugin_result.icon_name()).get());
        let name = plugin_result.name();

        imp.title.set_label(if name.len() > 300 {
            string_utils::truncate(name, 300)
        } else {
            name
        });

        match plugin_result.extra() {
            Some(desc) => {
                let desc = if desc.len() > 300 {
                    string_utils::truncate(desc, 300)
                } else {
                    desc
                };

                imp.extra.set_label(desc)
            }
            None => {
                imp.extra.hide();
            }
        }
    }

    pub fn unbind_all(&self) {
        let imp = self.imp();
        imp.image.clear();
    }
}
