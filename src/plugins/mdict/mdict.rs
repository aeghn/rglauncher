// Octopus MDict Dictionary File (.mdx) and Resource File (.mdd) Praser
//
// Copyright (C) 2012, 2013, 2015 Xiaoqiang Wang <xiaoqiangwang AT gmail DOT com>
// Copyright (C) 2020 韩朴宇 <w12101111@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/*!
This crate provides a library for parsing and lookup the Octopus MDict Dictionary
File (.mdx) and Resource File (.mdd).

MDict is a close source software, and don't provide any information about the format
of its file. But project [mdict-analysis](https://bitbucket.org/xwang/mdict-analysis/)
and [writemdict](https://github.com/zhansliu/writemdict) provide python library
to read and write MDict files based on reverse-engineering.
This crate is a rust rewrite of the python library `readmdict.py` from `mdict-analysis`.

A MDict Dictionary have one dictionary file and possible multiple (or none) resource files.
Those files have the same file name, extension name of dictionary file is `.mdx` and
extension of nth resource file is `.{n}.mdd` while the extension name of fist resource file
is just `.mdd`.For example, a valid MDict Dictionary have `a_dict.mdx`
or `b_dict.mdx`, `b_dict.mdd` and `b_dict.1.mdd`.

This is a low level crate for writing indexer of MDict Dictionary.
This crate can parse a mdx or mdx file, dump the keywords and record indexes and lookup the
record content of gaving record index.
A indexer will provide high level API to lookup keyword and resource in a MDict Dictionary.

For more information of the format of MDict file, see [mdict-analysis](https://bitbucket.org/xwang/mdict-analysis/)
and [fileformat](https://github.com/zhansliu/writemdict/blob/master/fileformat.md)

## Example

```
use std::fs::File;
use std::collections::HashMap;
use mdict::*;

fn main() -> std::io::Result<()> {
    let mut file = File::open("test.mdx")?;
    let mut mdict = MDictIndex::new(&mut file, MDictMode::Mdx)?;
    let (blocks, keys) = mdict.make_index()?;
    let header = mdict.into_header();
    let key_map: HashMap<String, MDictRecordIndex> = keys.into_iter().collect();
    match key_map.get("rust") {
        Some(idx) => {
            let record = lookup(file, idx, &blocks[idx.block as usize])?;
            let record = header.decode_string(record)?;
            println!("{}", record);
        }
        None => println!("Nothing found"),
    }
    Ok(())
}

```

*/

use bytes::{Buf, Bytes};
use encoding_rs::{Encoding, UTF_16LE};
use futures::AsyncSeekExt;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use regex::Regex;
use ripemd128::{Digest, Ripemd128};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::{self, prelude::*, Error, ErrorKind};
use tracing::{debug, info};

// The `Encrypted` field of MDict file header.
// The possible is 0, 1, 2, 3.
//
// If the lower bit is set, indicates that the header of keyword block is encrypted.
// This is checked in `read_keys` and passby in `search_key_block_index_size`
//
// If the upper bit is set, indicates that the index of keyword block is encrypted.
// This is checked in `read_keys` and decrypted in `decrypt_key_block_index`
#[derive(PartialEq)]
struct MDictEncryptionMode(u8);

// Prase from attribute `Encrypted` of MDict header
impl TryFrom<&str> for MDictEncryptionMode {
    type Error = io::Error;
    fn try_from(s: &str) -> io::Result<MDictEncryptionMode> {
        let mode = match s {
            "No" | "" => 0,
            "Yes" => 1,
            _ => s.parse().map_err(|_| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid or unknown encryption mode: {}", s),
                )
            })?,
        };
        Ok(MDictEncryptionMode(mode))
    }
}

impl MDictEncryptionMode {
    fn none() -> MDictEncryptionMode {
        MDictEncryptionMode(0)
    }
    fn mode(&self) -> u8 {
        self.0
    }
}

/// Indicated This Mdict File is `mdx` or `mdx`
///
/// There are two difference between `mdx` and `mdd`:
///
/// 1. The encoding of `mdd`'s keyword is always UTF-16LE, while
/// The encoding of `mdx` is specified in header's `Encoding` feild.
///
/// 2. The record of `mdx` is text or HTML, while the record of `mdd`
/// is compressed file.
#[derive(Clone, Copy, Debug)]
pub enum MDictMode {
    Mdx,
    Mdd,
}

/// There are some differences in file format between v1.2 and v2
/// v1 use 32 bit and 8 bit integer but v2 use 64 bit and 16 bit integer
/// to represent offset/size and length of string.
/// v2 also have a extra field in the header of key block
#[derive(Clone, Copy, Debug, PartialEq)]
enum MDictFormatVersion {
    V1,
    V2,
}

// Prase from attribute `GeneratedByEngineVersion` of MDict header
impl From<&str> for MDictFormatVersion {
    fn from(s: &str) -> MDictFormatVersion {
        let version: f32 = s.parse().unwrap_or(1.0);
        if version < 2.0 {
            MDictFormatVersion::V1
        } else {
            MDictFormatVersion::V2
        }
    }
}

/// The header of MDict file.
///
/// The header is originally a string of XML Tag.
/// Its attributes contains useful information such as "Title", "Description" and "CreationDate".
pub struct MDictHeader {
    encoding: &'static Encoding,
    encryption_mode: MDictEncryptionMode,
    version: MDictFormatVersion,
    /// Attributes of this header.
    pub attrs: HashMap<String, String>,
    /// This MDict file is a mdx or mdd file.
    pub mode: MDictMode,
}

impl MDictHeader {
    /// Prase MDict header from `reader`.
    ///
    /// **This reader should begin with content from a Mdict file.**
    ///
    /// After this function, the cursor of `reader` will stop at the end of the header.
    ///
    /// # Error
    ///
    /// This function returns [`io::Error`] if any io operations failed.
    ///
    /// [`io::Error`] with [`ErrorKind::InvalidData`] will return if the header is invalid, checksum is incorrect
    /// or can't be decoded to UTF-8.
    pub fn new<R: Read + Seek>(mut reader: R, mode: MDictMode) -> io::Result<MDictHeader> {
        reader.seek(io::SeekFrom::Start(0))?;
        let size = read_len(&mut reader, 4)?.as_slice().get_u32() as usize;
        let header_buf = read_len(&mut reader, size)?;
        let checksum = read_len(&mut reader, 4)?.as_slice().get_u32_le();
        let calc_checksum = adler::adler32_slice(&header_buf);
        check_eq(calc_checksum, checksum, "MDict header checksum")?;
        // two 0x0 in the end of the content
        let attrs = Self::parse_header(&header_buf[0..size - 2])?;
        info!("MDict header: {:#?}", attrs);
        let encoding = match mode {
            MDictMode::Mdx => Encoding::for_label(
                attrs
                    .get("Encoding")
                    .map(|s| s.as_str())
                    .unwrap_or("UTF-16LE")
                    .as_bytes(),
            )
            .unwrap_or(encoding_rs::UTF_16LE),
            MDictMode::Mdd => encoding_rs::UTF_16LE,
        };
        info!("Using encoding: {}", encoding.name());
        let encryption_mode = match attrs.get("Encrypted") {
            Some(e) => e.as_str().try_into()?,
            None => MDictEncryptionMode::none(),
        };
        let version = attrs
            .get("GeneratedByEngineVersion")
            .map(|e| e.as_str().into())
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Unknown format version: {:?}",
                        attrs.get("GeneratedByEngineVersion")
                    ),
                )
            })?;
        // TODO: deal with Title Description StyleSheet Compact Left2Right
        // StyleSheet format:
        // 3 lines per StyleSheet
        //   style_number # 1-255
        //   style_begin  # or ''
        //   style_end    # or ''
        // {'number' : ('style_begin', 'style_end')}
        // Title: meaningless title: "Title (No HTML code allowed)"
        Ok(MDictHeader {
            encoding,
            encryption_mode,
            version,
            attrs,
            mode,
        })
    }

    // parse the original XML tag from header and decode them into UTF-8
    fn parse_header(header_buf: &[u8]) -> io::Result<HashMap<String, String>> {
        let (cow, _encoding_used, had_errors) = UTF_16LE.decode(&header_buf);
        if had_errors {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Cannot decode MDict Header to UTF-8",
            ));
        }
        let re = Regex::new(r#"(\w+)="([^"]*?)""#).unwrap();
        let mut result = HashMap::new();
        for kv in re.captures_iter(&cow) {
            let key = kv[1].to_string();
            let val = html_escape::decode_html_entities(&kv[2]);
            result.insert(key, val.to_string());
        }
        Ok(result)
    }

    // The code unit size is the smallest size of char (in bytes) in this encoding
    fn unit_size(&self) -> usize {
        let name = self.encoding.name().to_ascii_lowercase();
        // Anyone still using BIG-5 ?
        if name.contains("utf-16") || name.contains("big5") {
            2
        } else {
            1
        }
    }

    /// Decode bytes into UTF-8 based on the encoding of this header.
    ///
    /// # Error
    ///
    /// [`io::Error`] with [`ErrorKind::InvalidData`] will return if src can't be decoded to UTF-8.
    pub fn decode_string(&self, src: Bytes) -> io::Result<String> {
        let (cow, _encoding_used, had_errors) = self.encoding.decode(&src);
        if had_errors {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("{} ({:X?}) cannot decode to UTF-8", cow, src),
            ))
        } else {
            Ok(String::from(cow))
        }
    }

    #[inline]
    fn version(&self) -> MDictFormatVersion {
        self.version
    }

    #[inline]
    /// get mode of This header.
    pub fn mode(&self) -> MDictMode {
        self.mode
    }

    #[inline]
    /// get the map of attributes of this header.
    pub fn attrs(&self) -> &HashMap<String, String> {
        &self.attrs
    }

    #[inline]
    /// get encoding of this header.
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
    }
}

/// A struct to build indexes from MDict file
///
/// This is a builder struct to dump keywords and indexes to records of corresponding keywords.
pub struct MDictIndex<R: Read + Seek> {
    file: io::BufReader<R>,
    key_block_offset: u64,
    header: MDictHeader,
}

/// A keywords block
///
/// Maybe the original auther of MDict want to use this as a index,
/// But a index only using `first_word` and `last_word` is inefficient even through the keywords is sorted.
/// So we only store this internally.
#[derive(Debug)]
struct MDictKeyBlockIndex {
    /// Number of keywords in this keyword block
    block_entries: u64,
    /// The first keyword in thos keyword block
    first_word: String,
    /// The last keyword in thos keyword block
    last_word: String,
    /// Compressed size of this keyword block
    comp_size: u64,
    /// Uncompressed size of this keyword block
    uncomp_size: u64,
    /// Words list, keyword and offset of its record
    /// from the begin of totally uncompressed record blcoks
    words: Vec<(String, u64)>,
}

/// Index to a compressed block which contains records
#[derive(Copy, Clone, Debug)]
pub struct MDictRecordBlockIndex {
    /// Offset of this record block from the start of the file
    pub offset: u64,
    /// Compressed size of this record block
    pub comp_size: u64,
}

/// Index to a record
#[derive(Copy, Clone, Debug)]
pub struct MDictRecordIndex {
    /// which block this record belongs to
    pub block: u32,
    /// the offset of this record from the start of the uncompressed block
    pub offset: u32,
    /// length of this record
    pub len: u32,
}

impl<R: Read + Seek> MDictIndex<R> {
    /// Build a new `MDictIndex`.
    ///
    /// **This reader should contain valid Mdict file.**
    ///
    /// This function will prase the header of this reader to ensure this is a valid MDict file.
    /// This function internally use a [`io::BufReader`], so no need to provide a `Reader`.
    ///
    /// # Error
    ///
    /// This function returns [`io::Error`] if any io operations failed just like [`MDictHeader`] do.
    pub fn new(reader: R, mode: MDictMode) -> io::Result<MDictIndex<R>> {
        let mut file = io::BufReader::with_capacity(0x10000, reader);
        let header = MDictHeader::new(&mut file, mode)?;
        let key_block_offset = file.seek(io::SeekFrom::Current(0))?;
        Ok(MDictIndex {
            file,
            key_block_offset,
            header,
        })
    }

    /// Read the keywords block.
    fn read_keys(&mut self) -> io::Result<Vec<MDictKeyBlockIndex>> {
        let unencrypted = self.header.encryption_mode.mode() & 0x1 == 0x0;
        let block_size = match self.header.version() {
            MDictFormatVersion::V1 => 4 * 4,
            MDictFormatVersion::V2 => 5 * 8,
        };
        let key_block_header = read_len(&mut self.file, block_size)?;
        if self.header.version() == MDictFormatVersion::V2 {
            let checksum = read_len(&mut self.file, 4)?.as_slice().get_u32();
            if unencrypted {
                let calc_checksum = adler::adler32_slice(&key_block_header);
                check_eq(calc_checksum, checksum, "Keywords block header checksum")?;
            }
        }
        // This closure will map those 5 number to None if header of key block is encrypted.
        let opt = |x| if unencrypted { Some(x) } else { None };
        let mut reader = key_block_header.as_slice();
        let key_block_num = opt(self.read_int(&mut reader));
        let entries_num = opt(self.read_int(&mut reader));
        let key_block_index_decomp_size = match self.header.version() {
            MDictFormatVersion::V1 => None,
            MDictFormatVersion::V2 => Some(opt(self.read_int(&mut reader))),
        };
        let key_block_index_size = opt(self.read_int(&mut reader));
        let key_block_size = opt(self.read_int(&mut reader));
        info!("number of entries: {:?}", entries_num);
        let now = std::time::Instant::now();
        let key_block_index_buf = match key_block_index_size {
            Some(size) => read_len(&mut self.file, size as usize)?,
            None => self.search_key_block_index_size()?,
        };
        let key_block_index_buf = match key_block_index_decomp_size {
            // v1
            None => key_block_index_buf.into(),
            // v2
            Some(decmp_size) => {
                let key_block_index_buf = if self.header.encryption_mode.mode() & 0x2 != 0 {
                    self.decrypt_key_block_index(key_block_index_buf)
                } else {
                    key_block_index_buf
                };
                let block = uncompress(key_block_index_buf.into()).unwrap();
                check_option_eq(
                    block.len() as u64,
                    decmp_size,
                    "Size of keywords block index",
                )?;
                block
            }
        };
        let key_block_index = self.read_key_block_index(key_block_index_buf)?;
        check_option_eq(
            key_block_index.len() as u64,
            key_block_num,
            "Number of keyword blocks",
        )?;
        let entries_calc: u64 = key_block_index.iter().map(|i| i.block_entries).sum();
        check_option_eq(
            entries_calc,
            entries_num,
            "Number entries in keywords block index",
        )?;
        let key_block_size_calc: u64 = key_block_index.iter().map(|i| i.comp_size).sum();
        check_option_eq(
            key_block_size_calc,
            key_block_size,
            "Size of keyword blocks",
        )?;
        info!("Decode keywords block index in {:?}", now.elapsed());
        let now = std::time::Instant::now();
        let key_block = read_len(&mut self.file, key_block_size_calc as usize)?.into();
        let keys = self.read_key_block(key_block, key_block_index)?;
        info!("Decode keywords blocks in {:?}", now.elapsed());
        Ok(keys)
    }

    /// Search magic number 0x{0,1,2},0x0,0x0,0x0 as start of keywords block
    fn search_key_block_index_size(&mut self) -> io::Result<Vec<u8>> {
        let now = std::time::Instant::now();
        // Ship possible magic number of keywords block index in v2
        let mut block = read_len(&mut self.file, 4)?;
        loop {
            self.file.read_until(0x0, &mut block)?;
            let next = read_len(&mut self.file, 2)?;
            if next.as_slice() == [0, 0]
                && block[block.len() - 1] == 0
                && block[block.len() - 2] <= 2
            {
                // return back 4 bytes magic number
                self.file.seek(io::SeekFrom::Current(-4))?;
                block.truncate(block.len() - 2);
                break;
            } else {
                block.extend(next);
            }
        }
        info!("Search end in {:?}", now.elapsed());
        Ok(block)
    }

    fn decrypt_key_block_index(&mut self, block: Vec<u8>) -> Vec<u8> {
        let mut key = Vec::from(&block[4..8]);
        key.extend(&0x3695u32.to_le_bytes());
        let mut hasher = Ripemd128::new();
        hasher.input(key);
        let hash_result = hasher.result();
        let key = hash_result.as_slice();
        let mut previous = 0x36;
        let mut result = vec![0; block.len()];
        result[..8].clone_from_slice(&block[..8]);
        let rest = &mut result[8..];
        for (i, v) in block.iter().skip(8).enumerate() {
            let mut t = (v >> 4) | (v << 4);
            t = t ^ previous ^ (i as u8) ^ key[i % key.len()];
            previous = *v;
            rest[i] = t;
        }
        result
    }

    fn read_key_block_index(&mut self, mut block: Bytes) -> io::Result<Vec<MDictKeyBlockIndex>> {
        let mut list = Vec::new();
        let unit_size = self.header.unit_size();
        // string in v2 end with unit_size \0
        let null_term = if self.header.version() == MDictFormatVersion::V2 {
            unit_size as usize
        } else {
            0
        };
        // Map the number of char to the real size in bytes.
        let map = |x| unit_size * x as usize + null_term;
        while !block.is_empty() {
            let block_entries = self.read_int(&mut block);
            let first_size = map(self.read_short(&mut block));
            let first_bytes = block.split_to(first_size);
            let first_word = self.header.decode_string(first_bytes)?;
            let last_size = map(self.read_short(&mut block));
            let last_bytes = block.split_to(last_size);
            let last_word = self.header.decode_string(last_bytes)?;
            let comp_size = self.read_int(&mut block);
            let uncomp_size = self.read_int(&mut block);
            list.push(MDictKeyBlockIndex {
                block_entries,
                first_word,
                last_word,
                comp_size,
                uncomp_size,
                // write in `read_key_block`
                words: Vec::with_capacity(block_entries as usize),
            });
        }
        Ok(list)
    }

    fn read_key_block(
        &mut self,
        mut block: Bytes,
        mut index: Vec<MDictKeyBlockIndex>,
    ) -> io::Result<Vec<MDictKeyBlockIndex>> {
        // basically strlen+strcpy, but support 2 bytes encoding like UTF-16LE
        let split_null = if self.header.unit_size() == 2 {
            split_dual_null
        } else {
            split_single_null
        };
        for idx in index.iter_mut() {
            let compressed = block.split_to(idx.comp_size as usize);
            let mut uncompressed = uncompress(compressed)?;
            check_eq(
                uncompressed.len() as u64,
                idx.uncomp_size,
                "Size of uncompressed content",
            )?;
            for _ in 0..idx.block_entries {
                let offset = self.read_int(&mut uncompressed);
                let string_encoded = split_null(&mut uncompressed);
                let string_decoded = self.header.decode_string(string_encoded)?;
                idx.words.push((string_decoded, offset));
            }
            if !uncompressed.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unexpected extra content at the end of keyword block".to_owned(),
                ));
            }
        }
        Ok(index)
    }

    /// Read keywords blocks and records blocks index, and generate the Index
    ///
    /// This function returns a `Vec` of `MDictRecordBlockIndex` and a `Vec` of `(String, MDictRecordIndex)`
    ///
    /// The `MDictRecordBlockIndex` is a index to the a record block of a MDict file.
    ///
    /// The `String` is the keyword and The `MDictRecordIndex` is a index to the record of this keyword.
    ///
    /// # Error
    ///
    /// This function returns [`io::Error`] if any operations failed.
    ///
    /// [`io::Error`] with [`ErrorKind::InvalidData`] will return if uncompression is failed, checksum is incorrect,
    /// length of blocks or header is incorrect or string can't be decoded to UTF-8.
    // TODO: Simplify return type
    pub fn make_index(
        &mut self,
    ) -> io::Result<(Vec<MDictRecordBlockIndex>, Vec<(String, MDictRecordIndex)>)> {
        self.file.seek(io::SeekFrom::Start(self.key_block_offset))?;
        // read keywords block is done in `read_keys`, this function is actually read record block index.
        let keys = self.read_keys()?;
        let header_size = match self.header.version() {
            MDictFormatVersion::V1 => 4 * 4,
            MDictFormatVersion::V2 => 4 * 8,
        };
        let header_buf = read_len(&mut self.file, header_size)?;
        let mut header = header_buf.as_slice();
        let num_blocks = self.read_int(&mut header);
        info!("record block num: {}", num_blocks);
        let num_entries = self.read_int(&mut header) as usize;
        let block_index_size = self.read_int(&mut header);
        info!("record block index size: {}", block_index_size);
        let blocks_size = self.read_int(&mut header);
        info!("record blocks size: {}", blocks_size);
        let block_index_size_calc = num_blocks
            * 2
            * match self.header.version() {
                MDictFormatVersion::V1 => 4,
                MDictFormatVersion::V2 => 8,
            };
        check_eq(
            block_index_size_calc,
            block_index_size,
            "Size of record block index",
        )?;
        let now = std::time::Instant::now();
        let block_index_bytes = read_len(&mut self.file, block_index_size as usize)?;
        let block_index = self.read_record_block_info(block_index_bytes.into())?;
        let blocks_size_calc: u64 = block_index.iter().map(|(c, _)| *c).sum();
        check_eq(blocks_size_calc, blocks_size, "Size of record block")?;
        info!("Decode record block index in {:?}", now.elapsed());

        let now = std::time::Instant::now();
        // collect pairs of (keywords, offset in uncompressed records), drop others
        let mut keys: Vec<_> = keys.into_iter().flat_map(|i| i.words.into_iter()).collect();
        // This should be already sorted.
        keys.sort_by_key(|(_, o)| *o);
        // take the start of record blocks
        let record_block_offset = self.file.seek(io::SeekFrom::Current(0))?;
        let mut indexes = Vec::with_capacity(num_entries as usize);
        let mut blocks = Vec::with_capacity(num_blocks as usize);
        let mut comp_offset = 0;
        let mut uncomp_offset = 0;
        let mut keys = keys.into_iter().peekable();
        for (bi, (comp_size, uncomp_size)) in block_index.into_iter().enumerate() {
            let record_block = MDictRecordBlockIndex {
                comp_size,
                offset: record_block_offset + comp_offset,
            };
            let next_comp_offset = comp_offset + comp_size;
            let next_uncomp_offset = uncomp_offset + uncomp_size;
            while let Some((key, o)) = keys.next() {
                let offset = o - uncomp_offset;
                assert!(offset <= uncomp_size);
                let end = match keys.peek() {
                    Some((_, next_offset)) => *next_offset,
                    None => next_uncomp_offset,
                };
                let len = end.max(next_comp_offset) - o;
                let index = MDictRecordIndex {
                    block: bi as u32,
                    offset: offset as u32,
                    len: len as u32,
                };
                indexes.push((key, index));
                if end >= next_uncomp_offset {
                    break;
                }
            }
            comp_offset = next_comp_offset;
            uncomp_offset = next_uncomp_offset;
            blocks.push(record_block);
        }
        info!("Generate index of keyword to record in {:?}", now.elapsed());
        Ok((blocks, indexes))
    }

    fn read_record_block_info(&mut self, mut block: Bytes) -> io::Result<Vec<(u64, u64)>> {
        let mut result = Vec::new();
        while !block.is_empty() {
            let comp_size = self.read_int(&mut block);
            let uncomp_size = self.read_int(&mut block);
            result.push((comp_size, uncomp_size));
        }
        Ok(result)
    }

    // get u32 in v1, u64 in v2
    fn read_int<B: Buf>(&self, buf: &mut B) -> u64 {
        match self.header.version() {
            MDictFormatVersion::V1 => buf.get_u32() as u64,
            MDictFormatVersion::V2 => buf.get_u64(),
        }
    }

    // get u8 in v1, u16 in v2
    fn read_short<B: Buf>(&self, buf: &mut B) -> u16 {
        match self.header.version() {
            MDictFormatVersion::V1 => buf.get_u8() as u16,
            MDictFormatVersion::V2 => buf.get_u16(),
        }
    }

    /// Consume this MDictIndex and return its header.
    ///
    /// This function is usually used after building the index to get the header, because after this,
    /// the reader, in some case a mutable reference to another `Read`, is released.
    /// The header also provides function to decode string and contains some information about the Dictionary.
    pub fn into_header(self) -> MDictHeader {
        self.header
    }
}

// read until one \0
fn split_single_null(buf: &mut Bytes) -> Bytes {
    for i in 0..buf.len() {
        if buf[i] == 0x0 {
            let string = buf.split_to(i);
            let _ = buf.split_to(1);
            return string;
        }
    }
    Bytes::new()
}

// read two bytes echo time until two \0
fn split_dual_null(buf: &mut Bytes) -> Bytes {
    if buf.len() > 2 {
        let mut i = 0;
        while i < buf.len() {
            if buf[i] == 0x0 && buf[i + 1] == 0x0 {
                let string = buf.split_to(i);
                let _ = buf.split_to(2);
                return string;
            }
            i += 2;
        }
    }
    Bytes::new()
}

fn check_option_eq(a: u64, b: Option<u64>, msg: &str) -> io::Result<()> {
    if let Some(b) = b {
        check_eq(a, b, msg)?;
    }
    Ok(())
}

fn check_eq<T: PartialEq + std::fmt::Display>(a: T, b: T, msg: &str) -> io::Result<()> {
    if a != b {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("{} mismatch: {} != {}", msg, a, b),
        ))
    } else {
        Ok(())
    }
}

// Uncompress block
fn uncompress(mut block: Bytes) -> io::Result<Bytes> {
    assert!(block.len() > 8);
    let magic = block.get_u32_le();
    let checksum = block.get_u32();
    let decompressed = match magic {
        0x0 => block,
        0x1 => minilzo::decompress(&block, 0x10000)
            .map_err(|e| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Lzo decompress failed: {:?}", e),
                )
            })?
            .into(),
        0x2 => decompress_to_vec_zlib(&block)
            .map_err(|e| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Lzo decompress failed: {:?}", e),
                )
            })?
            .into(),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown compression {:#X}", magic),
            ))
        }
    };
    let calc_checksum = adler::adler32_slice(&decompressed);
    check_eq(calc_checksum, checksum, "Checksum of uncompressed data")?;
    Ok(decompressed)
}

// read len bytes from this reader and return it as `Vec<u8>`
fn read_len<R: Read>(reader: &mut R, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(len);
    reader.take(len as u64).read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(feature = "async")]
use tokio::{io::AsyncSeek, prelude::*};

#[cfg(feature = "async")]
// read len bytes from this reader and return it as `Vec<u8>`
async fn read_len_async<R: AsyncRead + Unpin>(reader: &mut R, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(len);
    reader.take(len as u64).read_to_end(&mut buf).await?;
    Ok(buf)
}

#[cfg(not(feature = "async"))]
/// Lookup record of the given record index.
///
/// **This reader should contain valid Mdict file.**
///
/// The `key` should be corresponding to the keyword you want to lookup, and the `block` should be the
/// corresponding `MDictRecordBlockIndex` by the index of `key.block`.
///
/// The gaving `key` and `block` should be provided from `make_index` function, otherwise this lookup
/// may failed or return random data.
///
/// This is the blocking version of this function. To use asynchronous version, select the "async" crate feature
pub fn lookup<R>(
    reader: &mut R,
    key: &MDictRecordIndex,
    block: &MDictRecordBlockIndex,
) -> io::Result<Bytes>
where
    R: Read + Seek,
{
    reader.seek(io::SeekFrom::Start(block.offset))?;
    let compressed = read_len(reader, block.comp_size as usize)?;
    let comp_size = compressed.len();
    let mut uncompressed = uncompress(compressed.into())?;
    debug!(
        "uncompress record block {} -> {}",
        comp_size,
        uncompressed.len()
    );
    let mut data = uncompressed.split_off(key.offset as usize);
    data.truncate(key.len as usize);
    Ok(data)
}

#[cfg(feature = "async")]
/// Lookup record of the given record index.
///
/// **This reader should contain valid Mdict file.**
///
/// The `key` should be corresponding to the keyword you want to lookup, and the `block` should be the
/// corresponding `MDictRecordBlockIndex` by the index of `key.block`.
///
/// The gaving `key` and `block` should be provided from `make_index` function, otherwise this lookup
/// may failed or return random data.
///
/// This is the asynchronous version of this function. To use blocking version, unselect the "async" crate feature
pub async fn lookup<AR>(
    mut reader: AR,
    key: &MDictRecordIndex,
    block: &MDictRecordBlockIndex,
) -> io::Result<Bytes>
where
    AR: futures::AsyncRead + futures::AsyncSeek + Unpin,
{
    reader.seek(io::SeekFrom::Start(block.offset)).await?;
    let compressed = read_len_async(&mut reader, block.comp_size as usize).await?;
    let mut uncompressed = uncompress(compressed.into())?;
    let mut data = uncompressed.split_off(key.offset as usize);
    data.truncate(key.len as usize);
    Ok(data)
}
