use crate::icon_cache;
use crate::pluginpreview::PluginPreview;
use backend::plugins::windows::HyprWindowResult;
use backend::plugins::PluginResult;
use glib::Cast;
use gtk::pango::WrapMode::{Word, WordChar};
use gtk::prelude::{BoxExt, GridExt};
use gtk::Align::{Center, End};
use gtk::{Grid, Image, Orientation, Widget};

pub struct HyprWindowPreview {
    preview: gtk::Widget,
    big_pic: gtk::Image,
    little_pic: gtk::Image,
    title: gtk::Label,
    screen: gtk::Label,
    workspace: gtk::Label,
    xwayland: gtk::Label,
}

impl PluginPreview for HyprWindowPreview {
    type PluginResult = HyprWindowResult;

    fn new() -> Self {
        let b = gtk::Box::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(Center)
            .halign(Center)
            .orientation(Orientation::Vertical)
            .build();

        let big_pic = Image::builder()
            .icon_name("gnome-windows")
            .pixel_size(256)
            .vexpand(true)
            .build();

        let little_pic = gtk::Image::builder().pixel_size(64).hexpand(true).build();

        // preview.attach(&big_pic, 0, 0, 1, 1);
        // preview.attach(&little_pic, 0, 1, 1, 1);
        b.append(&big_pic);

        let title = gtk::Label::builder()
            .css_classes(["font16"])
            .wrap(true)
            .wrap_mode(WordChar)
            .selectable(true)
            .build();
        // preview.attach(&title, 0, 2, 1, 1);
        b.append(&title);

        let sep = super::get_seprator();
        let extra = gtk::Grid::builder()
            .hexpand(true)
            .vexpand(false)
            .valign(End)
            .build(); // preview.attach(&extra, 0, 3, 1, 1);

        let screen = super::build_pair_line(&extra, 0, "Screen: ");
        let workspace = super::build_pair_line(&extra, 1, "Workspace: ");
        let xwayland = super::build_pair_line(&extra, 2, "Xwayland: ");

        let sw = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        sw.set_child(Some(&b));

        let tb = gtk::Box::builder()
            .vexpand(true)
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();

        tb.append(&sw);
        tb.append(&sep);
        tb.append(&extra);

        HyprWindowPreview {
            preview: tb.upcast(),
            big_pic,
            little_pic,
            title,
            screen,
            workspace,
            xwayland,
        }
    }

    fn get_preview(&self) -> Widget {
        self.preview.clone().upcast()
    }

    fn set_preview(&self, plugin_result: &Self::PluginResult) {
        self.title.set_text(plugin_result.title.as_str());

        self.screen
            .set_label(plugin_result.monitor.to_string().as_str());
        self.workspace.set_label(&plugin_result.workspace);
        self.xwayland
            .set_label(plugin_result.xwayland.to_string().as_str());

        self.big_pic
            .set_from_gicon(icon_cache::get_icon(plugin_result.icon_name.as_str()).get());
    }

    fn get_id(&self) -> &str {
        backend::plugins::windows::TYPE_ID
    }
}
