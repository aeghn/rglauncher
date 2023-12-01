use crate::plugins::{Plugin, PluginResult};
use crate::user_input::UserInput;
use crate::util::{score_utils, string_utils};

use glib::{Cast};
use gtk::traits::{WidgetExt};

use fragile::Fragile;
use gtk::Widget;
use lazy_static::lazy_static;
use std::sync::Mutex;

use webkit6::traits::WebViewExt;
use webkit6::UserContentInjectedFrames::AllFrames;
use webkit6::UserStyleLevel::User;
use webkit6::{UserStyleSheet, WebView};

use self::mdx_utils::MDictLookup;

type DirType = String;

mod mdx_utils;
mod mdict;

lazy_static! {
    static ref PREVIEW: Mutex<Option<Fragile<webkit6::WebView>>> = Mutex::new(None);
}

pub struct DictPlugin {
    dir_path: String,
    mdxes: Vec<mdx_utils::MDictMemIndex>
}

pub struct DictPluginResult {
    word: String,
    html: String,
    pub dict: String,
}

impl DictPlugin {
    pub fn new(dir_path: &str) -> anyhow::Result<Self, anyhow::Error> {

        
       let mdxes: Vec<mdx_utils::MDictMemIndex> = std::fs::read_dir(dir_path)?
        .into_iter()
        .filter_map(|dr| {
            match dr {
                Ok(e) => {
                    let p = e.path();
                    Some(mdx_utils::MDictMemIndex::new(p).ok()?)
                },
                Err(x) => None,
            }
        }).collect();

        Ok(DictPlugin {
            dir_path: dir_path.to_string(),
            mdxes
        })

        
    }

    pub fn seek(&self, word: &str) -> Vec<DictPluginResult> {
        self.mdxes.iter()
        .filter_map(|mdx| {
            if let Ok(explain) = mdx.lookup_word(word) {
                Some(DictPluginResult{
                    word: word.to_string(), html: explain, dict: mdx.name.to_string() })
            } else {
                None
            }
        })
        .collect()
    }

    fn cycle_seek(&self, word: &str) -> Vec<DictPluginResult> {
        let w = word.trim();
        let seek_res = self.seek(w);

        let mut res: Vec<DictPluginResult> = vec![];
        for item in seek_res {
            if item.html.starts_with("@@@LINK=") {
                let w2 = item.html.replace("\r\n\0", "").replace("@@@LINK=", "");
                let r = self.cycle_seek(w2.as_str());
                res.extend(r);
            } else {
                res.push(item);
            }
        }

        res
    }
}

impl Plugin<DictPluginResult> for DictPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<DictPluginResult> {
        if user_input.input.is_empty() {
            return vec![];
        }

        self.cycle_seek(user_input.input.as_str())
    }
}

impl PluginResult for DictPluginResult {
    fn get_score(&self) -> i32 {
        return score_utils::middle(0);
    }

    fn sidebar_icon_name(&self) -> String {
        "dictionary".to_string()
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.word.to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(string_utils::truncate(self.dict.as_str(), 60).to_string())
    }

    fn preview(&self) -> Widget {
        let mut guard = PREVIEW.lock().unwrap();

        let wv = guard
            .get_or_insert_with(|| {
                let webview = WebView::new();
                webview.set_vexpand(true);
                webview.set_hexpand(true);
                webview.set_can_focus(false);

                if let Some(ucm) = webview.user_content_manager() {
                    ucm.remove_all_style_sheets();
                    let css = include_str!("../../../resources/dict.css");
                    let ss = UserStyleSheet::new(css, AllFrames, User, &[], &[]);
                    ucm.add_style_sheet(&ss);
                }

                Fragile::new(webview)
            })
            .get();

        let html_content = self.html
            .replace("\0", " ");
        wv.load_html(html_content.as_str(), None);

        wv.clone().upcast()
    }

    fn on_enter(&self) {
    }
}
