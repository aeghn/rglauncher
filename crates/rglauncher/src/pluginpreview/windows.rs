use crate::icon_cache;
use crate::pluginpreview::PluginPreview;
use backend::plugins::windows::HyprWindowResult;
use backend::plugins::PluginResult;
use glib::Cast;
use gtk::pango::WrapMode::{Word, WordChar};
use gtk::prelude::GridExt;
use gtk::Align::Center;
use gtk::{Grid, Image, Widget};

pub struct HyprWindowPreview {
    preview: gtk::Grid,
    big_pic: gtk::Image,
    little_pic: gtk::Image,
    title: gtk::Label,
    screen: gtk::Label,
    workspace: gtk::Label,
    xwayland: gtk::Label
}

fn build_pair_line(grid: &gtk::Grid, row: i32, title: &str) -> gtk::Label {
    let left = gtk::Label::builder()
            .xalign(1.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .label(title)
            .build();
    let right = gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .build();
    grid.attach(&left, 0, row, 1, 1);
    grid.attach(&right, 1, row, 1, 1);
    right
}

impl PluginPreview for HyprWindowPreview {
    type PluginResult = HyprWindowResult;

    fn new() -> Self {
        let preview = Grid::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(Center)
            .halign(Center)
            .build();

        let big_pic = Image::builder()
            .icon_name("gnome-windows")
            .pixel_size(256)
            .build();

        let little_pic = gtk::Image::builder().pixel_size(64).hexpand(true).build();

        preview.attach(&big_pic, 0, 0, 1, 1);
        preview.attach(&little_pic, 0, 1, 1, 1);

        let title = gtk::Label::builder()
            .css_classes(["font16"])
            .wrap(true)
            .wrap_mode(WordChar)
            .selectable(true)
            .build();
        preview.attach(&title, 0, 2, 1, 1);

        let extra = gtk::Grid::builder().hexpand(false).vexpand(false).halign(gtk::Align::Center).
        build();
        preview.attach(&extra, 0, 3, 1, 1);


        let screen = build_pair_line(&extra, 0, "Screen: ");
        let workspace = build_pair_line(&extra, 1, "Workspace: ");
        let xwayland = build_pair_line(&extra, 2, "Xwayland: ");


        HyprWindowPreview {
            preview,
            big_pic,
            little_pic,
            title,
            screen,
            workspace,
            xwayland
        }
    }

    fn get_preview(&self, plugin_result: &Self::PluginResult) -> Widget {
        self.title.set_text(plugin_result.title.as_str());

        self.screen.set_label(plugin_result.monitor.to_string().as_str());
        self.workspace.set_label(&plugin_result.workspace);
        self.xwayland.set_label(plugin_result.xwayland.to_string().as_str());

        self.big_pic
            .set_from_gicon(icon_cache::get_icon(plugin_result.icon_name.as_str()).get());

        self.preview.clone().upcast()
    }
}

