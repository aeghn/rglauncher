use super::PluginPreview;
use backend::plugins::dict::DictResult;
use glib::Cast;
use gtk::prelude::WidgetExt;
use gtk::Widget;
use webkit6::prelude::WebViewExt;
use webkit6::UserContentInjectedFrames::AllFrames;
use webkit6::UserStyleLevel::User;
use webkit6::{UserStyleSheet, WebView};

pub struct DictPreview {
    pub preview: WebView,
}

impl PluginPreview for DictPreview {
    type PluginResult = DictResult;
    fn new() -> Self {
        let webview = WebView::new();
        webview.set_vexpand(true);
        webview.set_hexpand(true);
        webview.set_can_focus(false);

        if let Some(ucm) = webview.user_content_manager() {
            ucm.remove_all_style_sheets();
            let css = include_str!("../../../../resources/dict.css");
            let ss = UserStyleSheet::new(css, AllFrames, User, &[], &[]);
            ucm.add_style_sheet(&ss);
        }

        DictPreview { preview: webview }
    }

    fn get_preview(&self, plugin_result: &DictResult) -> Widget {
        let html_content = plugin_result.html.replace("\0", " ");
        self.preview.load_html(html_content.as_str(), None);

        self.preview.clone().upcast()
    }
}
