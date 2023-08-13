
use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;
use crate::util::string_utils;
use futures::StreamExt;

use glib::Cast;
use gtk::traits::{StyleContextExt, WidgetExt};


use fragile::Fragile;
use gtk::Widget;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Mutex};


use webkit6::traits::WebViewExt;
use webkit6::UserContentInjectedFrames::AllFrames;
use webkit6::UserStyleLevel::User;
use webkit6::{UserStyleSheet, WebView};

type DirType = String;
type MdxPathType = String;

lazy_static! {
    static ref PREVIEW: Mutex<Option<Fragile<webkit6::WebView>>> = Mutex::new(None);
}

pub struct MDictPlugin {
    pub(crate) conn: Option<Connection>,
    map: HashMap<MdxPathType, DirType>,
    re: Regex,
}

pub struct MDictPluginResult {
    word: String,
    html: String,
    pub dict: String,
}

impl MDictPlugin {
    pub fn new(db_path: &str, files: Vec<MdxPathType>) -> Self {
        let conn = match Connection::open(db_path) {
            Ok(e) => Some(e),
            Err(_) => None,
        };

        let mut map: HashMap<MdxPathType, DirType> = HashMap::new();
        for mpt in files {
            let file = File::open(mpt.as_str());
            match file {
                Ok(_f) => match std::path::PathBuf::from(mpt.as_str()).parent() {
                    None => {}
                    Some(parent) => {
                        map.insert(mpt, parent.to_str().unwrap().to_string());
                    }
                },
                Err(_) => {}
            }
        }

        MDictPlugin {
            conn,
            map: Default::default(),
            re: Regex::new(r#"<link.*?>|<script.*?/script>"#).unwrap(),
        }
    }

    pub fn seek(&self, word: &str) -> Vec<(String, String, String)> {
        let sql = "select word, explain, dict, file_path from mdx_index where word = ?";
        if self.conn.is_none() {
            return vec![];
        }

        if let Some(_conn) = &self.conn {
            match _conn.prepare(sql) {
                Ok(mut e) => {
                    let iter = e.query_map(&[word], |row| {
                        let word = row.get(0).unwrap();
                        let explaination = row.get(1).unwrap();
                        let dict = row.get(2).unwrap();

                        Ok((word, explaination, dict))
                    });
                    let mut vec = vec![];
                    if let Ok(_iter) = iter {
                        for cpr in _iter {
                            vec.push(cpr.unwrap());
                        }
                    }
                    vec
                }
                Err(_) => {
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    fn cycle_seek(&self, word: &str) -> Vec<(String, String, String)> {
        let w = word.trim();
        let seek_res = self.seek(w);

        let mut res: Vec<(String, String, String)> = vec![];
        for (word, explanation, dict) in seek_res {
            if explanation.starts_with("@@@LINK=") {
                let w2 = explanation.replace("\r\n\0", "").replace("@@@LINK=", "");
                let r = self.cycle_seek(w2.as_str());
                res.extend(r);
            } else {
                res.push((word, explanation, dict))
            }
        }

        res
    }
}

impl Plugin<MDictPluginResult> for MDictPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<MDictPluginResult> {
        if user_input.input.is_empty() {
            return vec![];
        }

        let res = self.cycle_seek(user_input.input.as_str());
        res.into_iter()
            .map(|(word, explanation, dict)| MDictPluginResult {
                word,
                html: explanation.replace("\0", ""),
                dict,
            })
            .collect()
    }
}

impl PluginResult for MDictPluginResult {
    fn get_score(&self) -> i32 {
        return 100;
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

                if let Some(ucm) = webview.user_content_manager() {
                    ucm.remove_all_style_sheets();
                    let css = include_str!("../../../resources/dict.css");
                    let ss = UserStyleSheet::new(css, AllFrames, User, &[], &[]);
                    ucm.add_style_sheet(&ss);
                }

                Fragile::new(webview)
            })
            .get();

        let html_content = self.html.replace("\0", " ");
        wv.load_html(html_content.as_str(), None);

        wv.clone().upcast()
    }

    fn on_enter(&self) {
        todo!()
    }
}
