/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This is a Rust implementation of the [Mozilla Archive (MAR) file format][1]
//! used to deliver automatic updates to Firefox.  It includes both a library and
//! a command-line tool for reading and writing MAR files.
//!
//! This code is subject to the terms of the Mozilla Public License, v. 2.0.
//!
//! [1]: https://wiki.mozilla.org/Software_Update:MAR

#![warn(missing_docs)]

use std::{
    fs::File,
    io::{self, BufReader, Cursor, ErrorKind, Read, Seek, SeekFrom},
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt};
use compression::CompressedRead;
use read::{get_info, read_next_item};

pub mod compression;
pub mod extract;
pub mod read;

/// Metadata about an entire MAR file.
pub struct MarFileInfo {
    offset_to_index: u32,
    #[allow(dead_code)]
    has_signature_block: bool,
    #[allow(dead_code)]
    num_signatures: u32,
    #[allow(dead_code)]
    has_additional_blocks: bool,
    #[allow(dead_code)]
    offset_additional_blocks: u32,
    #[allow(dead_code)]
    num_additional_blocks: u32,
}

/// An entry in the MAR index.
pub struct MarItem {
    /// Position of the item within the archive file.
    offset: u32,
    /// Length of data in bytes.
    pub length: u32,
    /// File mode bits.
    pub flags: u32,
    /// File path.
    pub name: String,
}

/// A high level interface to read the contents of a mar file.
pub struct Mar<R> {
    info: MarFileInfo,
    buffer: R,
}

impl<R> Mar<R>
where
    R: Read + Seek,
{
    /// Creates a Mar instance from any seekable readable.
    pub fn from_buffer(mut buffer: R) -> io::Result<Mar<R>> {
        let info = get_info(&mut buffer)?;

        Ok(Mar { info, buffer })
    }
}

impl Mar<BufReader<File>> {
    /// Creates a Mar instance from a local file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Mar<BufReader<File>>> {
        let buffer = BufReader::new(File::open(path)?);
        Self::from_buffer(buffer)
    }
}

impl<R> Mar<R>
where
    R: Read + Seek,
{
    /// Reads the contents of a file from this mar.
    pub fn read<'a>(&'a mut self, item: &MarItem) -> io::Result<CompressedRead<'a, R>> {
        self.buffer.seek(SeekFrom::Start(item.offset as u64))?;
        CompressedRead::new(&mut self.buffer, item.length as u64)
    }

    /// Returns an Iterator to the list of files in this mar.
    pub fn files(&mut self) -> io::Result<Files> {
        self.buffer
            .seek(SeekFrom::Start(self.info.offset_to_index as u64))?;

        // Read the index into memory.
        let size_of_index = self.buffer.read_u32::<BigEndian>()?;
        let mut index = vec![0; size_of_index as usize];
        self.buffer.read_exact(&mut index)?;

        Ok(Files {
            index: Cursor::new(index),
        })
    }
}

/// An iterator over the files in a mar.
pub struct Files {
    index: Cursor<Vec<u8>>,
}

impl Iterator for Files {
    type Item = io::Result<MarItem>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_next_item(&mut self.index) {
            Ok(item) => Some(Ok(item)),
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    None
                } else {
                    Some(Err(e))
                }
            }
        }
    }
}
