use crate::iconcache;
use crate::pluginpreview::PluginPreview;
use backend::plugins::app::AppResult;
use glib::Cast;
use gtk::prelude::GridExt;

pub struct AppPreview {
    root: gtk::Widget,
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
            .css_classes(["preview-big-font"])
            .wrap(true)
            .build();
        preview.attach(&name, 0, 1, 1, 1);

        let desc = gtk::Label::builder().wrap(true).build();
        preview.attach(&desc, 0, 2, 1, 1);

        let exec = gtk::Label::builder()
            .wrap(true)
            .css_classes(["dim-label"])
            .build();
        preview.attach(&exec, 0, 3, 1, 1);

        let w = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        w.set_child(Some(&preview));

        AppPreview {
            root: w.upcast(),
            icon,
            name,
            desc,
            exec,
        }
    }

    fn get_preview(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn set_preview(&self, plugin_result: &Self::PluginResult) {
        self.icon
            .set_from_gicon(iconcache::get_icon(plugin_result.icon_name.as_str()).get());
        self.name.set_label(plugin_result.app_name.as_str());
        self.exec.set_label(plugin_result.desktop_path.as_str());
        self.desc.set_label(plugin_result.app_desc.as_str());
    }

    fn get_id(&self) -> &str {
        backend::plugins::app::TYPE_ID
    }
}
