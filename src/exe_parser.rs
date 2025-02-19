/*
 * Original code is from doukutsu-rs.
 *
 * MIT/doukutsu-rs License

 * Copyright 2020 doukutsu-rs contributors.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software
 * and associated documentation files (the "Software"), to deal in the Software without restriction,
 * including without limitation the rights to use, copy, modify, merge, publish, distribute,
 * sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or
 * substantial portions of the Software.
 *
 * The Software cannot be redistributed bundled with data files taken from any commercial port
 * released by Nicalis Inc. without their explicit permission.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
 * BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
 * DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::ops::Range;

use pelite::{
    image::RT_BITMAP,
    pe32::{headers::SectionHeaders, Pe, PeFile},
    resources::{Directory, Entry, Name, Resources},
};

use crate::error::{GameError::ParseError, GameResult};

#[derive(Debug)]
pub struct DataFile {
    pub bytes: Vec<u8>,
    pub name: String,
}

impl DataFile {
    pub fn from(name: String, bytes: Vec<u8>) -> Self {
        Self { name, bytes }
    }
}

#[derive(Debug)]
pub struct ExeResourceDirectory {
    pub name: String,
    pub data_files: Vec<DataFile>,
}

impl ExeResourceDirectory {
    pub fn new(name: String) -> Self {
        Self { name, data_files: Vec::new() }
    }
}

pub struct ExeParser<'a> {
    pub image_base: u32,
    pub resources: Resources<'a>,
    pub section_headers: Box<&'a SectionHeaders>,
}

impl<'a> ExeParser<'a> {
    pub fn from(file: &'a Vec<u8>) -> GameResult<Self> {
        let pe = PeFile::from_bytes(file);

        return match pe {
            Ok(pe) => {
                let resources = pe.resources();

                if resources.is_err() {
                    return Err(ParseError("Failed to parse resources.".to_string()));
                }

                let section_headers = pe.section_headers();
                let image_base = pe.nt_headers().OptionalHeader.ImageBase;

                Ok(Self {
                    image_base,
                    resources: resources.unwrap(),
                    section_headers: Box::new(section_headers)
                })
            }
            Err(_) => Err(ParseError("Failed to parse PE file".to_string())),
        };
    }

    pub fn get_resource_dir(&self, name: String) -> GameResult<ExeResourceDirectory> {
        let mut dir_data = ExeResourceDirectory::new(name.to_owned());

        let path = format!("/{}", name.to_owned());
        let dir = self.resources.find_dir(&path);

        return match dir {
            Ok(dir) => {
                self.read_dir(dir, &mut dir_data, "unknown".to_string());
                Ok(dir_data)
            }
            Err(_) => return Err(ParseError("Failed to find resource directory.".to_string())),
        };
    }

    pub fn get_bitmap_dir(&self) -> GameResult<ExeResourceDirectory> {
        let mut dir_data = ExeResourceDirectory::new("Bitmap".to_string());

        let root = self.resources.root().unwrap();
        let dir = root.get_dir(Name::Id(RT_BITMAP.into()));

        return match dir {
            Ok(dir) => {
                self.read_dir(dir, &mut dir_data, "unknown".to_string());
                Ok(dir_data)
            }
            Err(_) => return Err(ParseError("Failed to open bitmap directory.".to_string())),
        };
    }

    pub fn get_named_section_byte_range(&self, name: String) -> GameResult<Option<Range<u32>>> {
        let section_header = self.section_headers.by_name(name.as_bytes());
        return match section_header {
            Some(section_header) => Ok(Some(section_header.file_range())),
            None => Ok(None),
        };
    }

    fn read_dir(&self, directory: Directory, dir_data: &mut ExeResourceDirectory, last_dir_name: String) {
        for dir in directory.entries() {
            let raw_entry = dir.entry();

            if raw_entry.is_err() {
                continue;
            }

            if let Entry::Directory(entry) = raw_entry.unwrap() {
                let dir_name = dir.name();
                let name = match dir_name {
                    Ok(name) => name.to_string(),
                    Err(_) => last_dir_name.to_owned(),
                };
                self.read_dir(entry, dir_data, name);
            }

            if let Entry::DataEntry(entry) = raw_entry.unwrap() {
                let entry_bytes = entry.bytes();
                if entry_bytes.is_err() {
                    continue;
                }

                let bytes = entry_bytes.unwrap();
                let data_file = DataFile::from(last_dir_name.to_owned(), bytes.to_vec());
                dir_data.data_files.push(data_file);
            }
        }
    }
}
