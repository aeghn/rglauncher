use crate::plugins::mdict::mdict::{
    MDictIndex, MDictMode, MDictRecordBlockIndex, MDictRecordIndex,
};
use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;
use crate::util::string_utils;
use futures::StreamExt;
use gio::Icon;
use glib::{Cast, StrV};
use gtk::traits::{GridExt, StyleContextExt, WidgetExt};
use gtk::AccessibleRole::Label;
use gtk::{Align, Grid, Widget};
use regex::Regex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::process::id;
use std::sync::Arc;
use tracing::error;
use webkit6::traits::WebViewExt;
use webkit6::UserContentInjectedFrames::AllFrames;
use webkit6::UserStyleLevel::User;
use webkit6::{UserContentManager, UserStyleSheet, WebView};

mod mdict;

pub struct Index {
    word: String,
    block: MDictRecordBlockIndex,
    idx: MDictRecordIndex,
}

type DirType = String;
type MdxPathType = String;

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
                Ok(f) => match std::path::PathBuf::from(mpt.as_str()).parent() {
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
            re: Regex::new(r#"<link.*?/>|<script.*?/script>"#).unwrap(),
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
            let explanation = self.re.replace_all(explanation.as_str(), "").to_string();
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

    fn sidebar_icon(&self) -> Option<Icon> {
        Some(gio::Icon::from(gio::ThemedIcon::from_names(&[
            &"dictionary",
        ])))
    }

    fn sidebar_label(&self) -> Option<String> {
        Some(self.word.to_string())
    }

    fn sidebar_content(&self) -> Option<String> {
        Some(string_utils::truncate(self.dict.as_str(), 60).to_string())
    }

    fn preview(&self) -> Widget {
        let webview = WebView::new();
        webview.set_vexpand(true);
        webview.set_hexpand(true);

        let context = webview.user_content_manager();
        if let Some(ucm) = context {
            ucm.remove_all_style_sheets();
            let css = include_str!("../../../resources/dict.css");
            let ss = UserStyleSheet::new(css, AllFrames, User, &[], &[]);
            ucm.add_style_sheet(&ss);
        }

        // Load HTML content
        let mut html_content = self.html.replace("\0", " ");
        webview.load_html(html_content.as_str(), None);

        webview.upcast()
    }

    fn on_enter(&self) {
        todo!()
    }
}
