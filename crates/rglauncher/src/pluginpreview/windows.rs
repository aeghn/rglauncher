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
    extra: gtk::Label,
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
            .build();
        preview.attach(&title, 0, 2, 1, 1);

        let extra = gtk::Label::builder()
            .wrap(true)
            .wrap_mode(Word)
            .hexpand(true)
            .build();
        preview.attach(&extra, 0, 3, 1, 1);

        HyprWindowPreview {
            preview,
            big_pic,
            little_pic,
            title,
            extra,
        }
    }

    fn get_preview(&self, plugin_result: &Self::PluginResult) -> Widget {
        self.title.set_text(plugin_result.title.as_str());

        if let Some(c) = plugin_result.sidebar_content() {
            self.extra.set_text(c.as_str());
        }

        self.big_pic
            .set_from_gicon(icon_cache::get_icon(plugin_result.icon_name.as_str()).get());

        self.preview.clone().upcast()
    }
}
