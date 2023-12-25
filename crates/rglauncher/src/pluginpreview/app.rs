use crate::icon_cache;
use crate::pluginpreview::PluginPreview;
use backend::plugins::app::AppResult;
use glib::Cast;
use gtk::prelude::GridExt;

pub struct AppPreview {
    root: gtk::Grid,
    icon: gtk::Image,
    name: gtk::Label,
    desc: gtk::Label,
    exec: gtk::Label,
}

impl PluginPreview for AppPreview {
    type PluginResult = AppResult;
    fn new() -> Self {
        let preview = gtk::Grid::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

        let icon = gtk::Image::builder().pixel_size(256).build();
        preview.attach(&icon, 0, 0, 1, 1);

        let name = gtk::Label::builder()
            .css_classes(["font32"])
            .wrap(true)
            .build();
        preview.attach(&name, 0, 1, 1, 1);

        let desc = gtk::Label::builder().wrap(true).build();
        preview.attach(&desc, 0, 2, 1, 1);

        let exec = gtk::Label::builder().wrap(true).build();
        preview.attach(&exec, 0, 3, 1, 1);

        AppPreview {
            root: preview,
            icon,
            name,
            desc,
            exec,
        }
    }

    fn get_preview(&self, plugin_result: &AppResult) -> gtk::Widget {
        self.icon
            .set_from_gicon(icon_cache::get_icon(plugin_result.app_name.as_str()).get());
        self.name.set_label(plugin_result.app_name.as_str());
        self.exec.set_label(plugin_result.exec_path.as_str());
        self.desc.set_label(plugin_result.app_desc.as_str());

        self.root.clone().upcast()
    }
}