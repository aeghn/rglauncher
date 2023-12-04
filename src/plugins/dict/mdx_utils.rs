use bytes::Bytes;
use mdict::*;
use patricia_tree::PatriciaMap;
use std::{
    fs::OpenOptions,
    io,
    path::{Path, PathBuf},
};

use super::mdict;

pub trait MDictLookup {
    fn word_exists(&self, key: &str) -> io::Result<bool>;
    fn lookup_word(&self, key: &str) -> io::Result<String>;
    fn lookup_resource(&self, key: &str) -> io::Result<Bytes>;
}

pub trait MDictAsyncLookup {
    fn word_exists(&self, key: &str) -> io::Result<bool>;
    fn lookup_word(&self, key: &str) -> io::Result<String>;
    fn lookup_resource(&self, key: &str) -> io::Result<Bytes>;
}

pub struct MDictMemIndex {
    mdx_index: PatriciaMap<MDictRecordIndex>,
    mdx_block: Vec<MDictRecordBlockIndex>,
    mdx_file: PathBuf,
    mdd_index: PatriciaMap<(u8, MDictRecordIndex)>,
    mdd_blocks: Vec<Vec<MDictRecordBlockIndex>>,
    mdd_files: Vec<PathBuf>,
    header: MDictHeader,
    pub name: String,
}

impl MDictMemIndex {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<MDictMemIndex> {
        let mdx_file = path.as_ref().canonicalize()?;
        if !mdx_file.is_file()
            || mdx_file
                .extension()
                .map(|s| s.to_str())
                .flatten()
                .map(|s| s.to_ascii_lowercase())
                != Some(String::from("mdx"))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Expect a mdx file",
            ));
        }
        tracing::debug!("mdx: {}", mdx_file.to_string_lossy());
        let mut mdd_files = Vec::new();
        let mdd0 = mdx_file.with_extension("mdd");
        if mdd0.is_file() {
            mdd_files.push(mdd0);
            for i in 1.. {
                let ext = format!("{}.mdd", i);
                let mddi = mdx_file.with_extension(ext);
                if mddi.is_file() {
                    tracing::debug!("mdd: {}", mddi.to_string_lossy());
                    mdd_files.push(mddi);
                } else {
                    break;
                }
            }
        }
        let mut mdx = MDictIndex::new(
            OpenOptions::new().read(true).open(&mdx_file)?,
            MDictMode::Mdx,
        )?;
        let (mdx_block, mdx_keys) = mdx.make_index()?;
        let now = std::time::Instant::now();
        let mdx_index = mdx_keys.into_iter().collect();
        tracing::debug!("Build Patricia Map for mdx in {:?}", now.elapsed());
        let mut mdd_index = PatriciaMap::new();
        let mut mdd_blocks = Vec::new();
        for (i, file) in mdd_files.iter().enumerate() {
            let mut mdd = MDictIndex::new(
                OpenOptions::new().read(true).clone().open(file)?,
                MDictMode::Mdd,
            )?;
            let (mdd_block, mdd_keys) = mdd.make_index()?;
            let now = std::time::Instant::now();
            mdd_index.extend(mdd_keys.into_iter().map(|(k, idx)| {
                // process keys when building map rather than lookup
                let (prefix, key) = k.split_at(1);
                assert_eq!(prefix, "\\");
                let key = key.replace('\\', "/");
                (key, (i as u8, idx))
            }));
            mdd_blocks.push(mdd_block);
            tracing::debug!("Build Patricia Map for mdd {} in {:?}", i, now.elapsed());
        }
        Ok(MDictMemIndex {
            mdx_index,
            mdx_block,
            mdx_file,
            mdd_index,
            mdd_blocks,
            mdd_files,
            header: mdx.into_header(),
            name: "".to_string()
        })
    }
    pub fn keyword_iter(&self) -> impl Iterator<Item = String> + '_ {
        self.mdx_index.keys().map(|k| String::from_utf8(k).unwrap())
    }
}

impl MDictLookup for MDictMemIndex {
    fn word_exists(&self, key: &str) -> io::Result<bool> {
        Ok(self.mdx_index.get(&key).is_some())
    }
    fn lookup_word(&self, key: &str) -> io::Result<String> {
        match self.mdx_index.get(&key) {
            Some(idx) => {
                let file = OpenOptions::new().read(true).open(&self.mdx_file)?;
                let bytes = lookup(file, idx, &self.mdx_block[idx.block as usize])?;
                let decoded = self.header.decode_string(bytes)?;
                Ok(decoded)
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Not found in index",
            )),
        }
    }

    fn lookup_resource(&self, key: &str) -> io::Result<Bytes> {
        match self.mdd_index.get(key) {
            Some((num, idx)) => {
                let file = OpenOptions::new()
                    .read(true)
                    .open(&self.mdd_files[*num as usize])?;
                let data = lookup(
                    file,
                    idx,
                    &self.mdd_blocks[*num as usize][idx.block as usize],
                )?;
                Ok(data)
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Not found in index",
            )),
        }
    }
}