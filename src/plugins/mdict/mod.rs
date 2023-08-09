use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::process::id;
use gio::Icon;
use gtk::Grid;
use rusqlite::Connection;
use tracing::error;
use crate::plugins::mdict::mdict::{MDictIndex, MDictMode, MDictRecordBlockIndex, MDictRecordIndex};
use crate::plugins::{Plugin, PluginResult};
use crate::shared::UserInput;

mod mdict;

pub struct Index {
    word: String,
    block: MDictRecordBlockIndex,
    idx: MDictRecordIndex
}

type DirType = String;
type MdxPathType = String;

pub struct MDictPlugin {
    pub(crate) conn: Option<Connection>,
    map: HashMap<MdxPathType, (File, DirType)>
}

pub struct MDictPluginResult {
    html: String,
}

impl MDictPlugin {
    pub fn new(db_path: &str,
               files: Vec<MdxPathType>) -> Self {
        let conn = match Connection::open(db_path) {
            Ok(e) => Some(e),
            Err(_) => None,
        };

        let mut map: HashMap<MdxPathType, (File, DirType)> = HashMap::new();
        for mpt in files {
            let file = File::open(mpt.as_str());
            match file {
                Ok(f) => {
                    match std::path::PathBuf::from(mpt.as_str()).parent() {
                        None => {}
                        Some(parent) => {
                            map.insert(mpt, (f, parent.to_str().unwrap().to_string()));
                        }
                    }

                }
                Err(_) => {}
            }

        }

        MDictPlugin {
            conn,
            map: Default::default(),
        }
    }

    pub fn save_into_db(&self, dict_dirs: Vec<String>, db_path: String) -> std::io::Result<()> {
        let conn = match Connection::open(db_path) {
            Ok(e) => Some(e),
            Err(_) => None,
        };

        if conn.is_none() {
            return Ok(());
        }

        self.create_table(conn.as_ref());

        for dir in dict_dirs {
            let path = std::path::Path::new(dir.as_str());
            if path.exists() {
                if path.is_dir() {
                    let entries = std::fs::read_dir(path).unwrap();
                    for x in entries {
                        if let Ok(en) = x {
                            if let Some(name) = en.path().to_str() {
                                if name.to_lowercase().ends_with("mdx") {
                                    let mut file = File::open(name)?;
                                    let mut mdict = MDictIndex::new(&mut file, MDictMode::Mdx)?;
                                    let (blocks, keys) = mdict.make_index()?;
                                    let header = mdict.into_header();
                                    for (word, idx) in keys {
                                        let record = mdict::lookup(&mut file, &idx, &blocks[idx.block as usize])?;
                                        let record = header.decode_string(record)?;
                                        if record.contains("<span class") {
                                            continue;
                                        }
                                        self.insert_one_item(conn.as_ref(), word.as_str(), record.as_str(), name);
                                        error!("insert {:?}", word.as_str());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn insert_one_item(&self, conn: Option<&Connection>,
                       word: &str,
                       text: &str,
                       file_path: &str) {
        let stmt = conn.unwrap().prepare("insert into mdx_index(word, explain, file_path) values (?, ?, ?)");

        match stmt {
            Ok(mut stmt) => {
                stmt.execute(&[word,
                    text,
                    file_path]).expect("TODO: panic message");
            }
            Err(e) => {
                error!("unable to create table: {:?}", e)
            }
        }
    }

    fn create_table(&self, conn: Option<&Connection>) {
        let sql = "
        CREATE TABLE mdx_index (
        word text not null,
        explain text not null,
        file_path text
        );";
        if let Some(con) = conn {
            con.execute("drop table if exists mdx_index;", []).unwrap();
            con.execute(sql, []).unwrap();
            con.execute("CREATE INDEX idx_word ON mdx_index (word);", []).unwrap();
        }
    }

    // pub fn seek(&self, word: &str) -> Vec<(String, String)>{
    //     let sql = "select word, explain, file_path from mdx_index where word = ?";
    //     if self.conn.is_none() {
    //         return vec![];
    //     }
    //
    //     let stmt = self.conn.unwrap().prepare(sql);
    //     match stmt {
    //         Ok(mut e) => {
    //             e.query_map(&[word], |row| -> {
    //                 let word = row.get(0).unwrap();
    //                 let file_path = row.get(7).unwrap();
    //                 match self.map.get(file_path) {
    //                     None => {}
    //                     Some(file, dir) => {
    //                         let result = mdict::lookup2(file,
    //                         row.get(1).unwrap(),
    //                             row.get(2).unwrap(),
    //                             row.get(3).unwrap(),
    //                             row.get(4).unwrap()
    //                         );
    //                     }
    //                 }
    //             })
    //         }
    //         Err(_) => {}
    //     }
    // }
}

impl Plugin<MDictPluginResult> for MDictPlugin {
    fn handle_input(&self, user_input: &UserInput) -> Vec<MDictPluginResult> {
        todo!()
    }
}

impl PluginResult for MDictPluginResult {
    fn get_score(&self) -> i32 {
        todo!()
    }

    fn sidebar_icon(&self) -> Option<Icon> {
        todo!()
    }

    fn sidebar_label(&self) -> Option<String> {
        todo!()
    }

    fn sidebar_content(&self) -> Option<String> {
        todo!()
    }

    fn preview(&self) -> Grid {
        todo!()
    }

    fn on_enter(&self) {
        todo!()
    }
}