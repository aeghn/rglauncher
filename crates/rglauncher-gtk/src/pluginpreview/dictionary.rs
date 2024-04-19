use super::PluginPreview;
use rglcore::plugins::dictionary::DictResult;

use glib::object::Cast;
use gtk::prelude::WidgetExt;
use gtk::Widget;
use webkit6::prelude::WebViewExt;
use webkit6::UserContentInjectedFrames::AllFrames;
use webkit6::UserStyleLevel::User;
use webkit6::{UserStyleSheet, WebView};

pub struct DictPreview {
    pub webview: WebView,
}

impl DictPreview {
    pub fn add_csses(&self, dirpath: &str) {
        if let Some(ucm) = self.webview.user_content_manager() {
            ucm.remove_all_style_sheets();
            let paths = rglcore::util::fs_utils::walk_dir(
                dirpath,
                Some(|p: &str| p.to_lowercase().ends_with(".css")),
            );

            if let Ok(des) = paths {
                for de in des {
                    let css = std::fs::read_to_string(de.path()).expect("unable to read file");
                    let ss = UserStyleSheet::new(css.as_str(), AllFrames, User, &[], &[]);
                    ucm.add_style_sheet(&ss);
                }
            }
        }

        self.webview.set_css_classes(&["webview"]);
    }
}

impl PluginPreview for DictPreview {
    type PluginResult = DictResult;

    fn new() -> Self {
        let webview = WebView::new();
        webview.set_vexpand(true);
        webview.set_hexpand(true);
        webview.set_focusable(false);

        DictPreview { webview }
    }

    fn get_preview(&self) -> Widget {
        self.webview.clone().upcast()
    }

    fn set_preview(&self, plugin_result: &Self::PluginResult) {
        let html_content = plugin_result.html.replace("\0", " ");
        self.webview.load_html(html_content.as_str(), None);
        self.webview
            .set_background_color(&gtk::gdk::RGBA::new(0., 0., 0., 0.));
    }

    fn get_id(&self) -> &str {
        rglcore::plugins::dictionary::TYPE_ID
    }
}
