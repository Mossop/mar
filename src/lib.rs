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

use std::{
    fs::File,
    io::{self, BufReader, Cursor, ErrorKind, Read, Seek, SeekFrom, Take},
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt};
use read::{get_info, read_next_item};

pub mod extract;
pub mod read;

/// Metadata about an entire MAR file.
pub struct MarFileInfo {
    offset_to_index: u32,
    has_signature_block: bool,
    num_signatures: u32,
    has_additional_blocks: bool,
    offset_additional_blocks: u32,
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

/// Round `n` up to the nearest multiple of `incr`.
#[inline]
fn round_up(n: usize, incr: usize) -> usize {
    n / (incr + 1) * incr
}

/// Make sure the file is less than 500MB.  We do this to protect against invalid MAR files.
const MAX_SIZE_OF_MAR_FILE: u64 = 500 * 1024 * 1024;

/// The maximum size of any signature supported by current and future implementations of the
/// signmar program.
const MAX_SIGNATURE_LENGTH: usize = 2048;

/// Each additional block has a unique ID.  The product information block has an ID of 1.
const PRODUCT_INFO_BLOCK_ID: u32 = 1;

/// An index entry contains three 4-byte fields, a name, and a 1-byte terminator.
///
/// * 4 bytes : OffsetToContent - Offset in bytes relative to start of the MAR file
/// * 4 bytes : ContentSize - Size in bytes of the content
/// * 4 bytes : Flags - File permission bits (in standard unix-style format).
/// * M bytes : FileName - File name (byte array)
/// * 1 byte  : null terminator
#[inline]
fn mar_item_size(name_len: usize) -> usize {
    3 * 4 + name_len + 1
}

struct ProductInformationBlock {
    mar_channel_id: Vec<u8>,
    product_version: Vec<u8>,
}

// Product Information Block (PIB) constants:
const PIB_MAX_MAR_CHANNEL_ID_SIZE: usize = 63;
const PIB_MAX_PRODUCT_VERSION_SIZE: usize = 31;

pub struct Mar<R> {
    info: MarFileInfo,
    buffer: R,
}

impl<R> Mar<R> {
    pub fn from_buffer<T: Read + Seek>(mut buffer: T) -> io::Result<Mar<T>> {
        let info = get_info(&mut buffer)?;

        Ok(Mar { info, buffer })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Mar<BufReader<File>>> {
        let buffer = BufReader::new(File::open(path)?);
        Self::from_buffer(buffer)
    }
}

impl<R> Mar<R>
where
    R: Read + Seek,
{
    pub fn read<'a>(&'a mut self, item: &MarItem) -> io::Result<Take<&mut R>> {
        self.buffer.seek(SeekFrom::Start(item.offset as u64))?;
        Ok(self.buffer.by_ref().take(item.length as u64))
    }

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
