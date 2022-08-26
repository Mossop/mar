/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Handles decompressing the file data within the mar.

use std::io::{self, ErrorKind, Read, Seek, Take};

use xz::read::XzDecoder;

const BZ2_HEADER: [u8; 3] = [b'B', b'Z', b'h'];
const XZ_HEADER: [u8; 6] = [253, b'7', b'z', b'X', b'Z', 0];

enum Compression<'a, R>
where
    R: Read + Seek,
{
    None(Take<&'a mut R>),
    Xz(XzDecoder<Take<&'a mut R>>),
}

/// A decompressing wrapper around another Read implementation.
pub struct CompressedRead<'a, R>
where
    R: Read + Seek,
{
    compression: Compression<'a, R>,
}

impl<'a, R> CompressedRead<'a, R>
where
    R: Read + Seek,
{
    /// Creates a decompressing wrapper around the given Read implementation.
    ///
    /// Attempts to autodetect the type of compression in use, currently XZ is
    /// the only format supported.
    pub fn new(inner: &'a mut R, length: u64) -> io::Result<CompressedRead<'a, R>> {
        let position = inner.stream_position()?;

        let mut header = [0_u8; 6];

        if length > 6 {
            inner.read_exact(&mut header)?;
        } else if length > 3 {
            inner.read_exact(&mut header[0..3])?;
        }

        inner.seek(io::SeekFrom::Start(position))?;

        if header[0..3] == BZ2_HEADER {
            Err(io::Error::new(
                ErrorKind::InvalidData,
                "BZ2 compression not yet supported.",
            ))
        } else if header == XZ_HEADER {
            Ok(Self {
                compression: Compression::Xz(XzDecoder::new(inner.take(length))),
            })
        } else {
            Ok(Self {
                compression: Compression::None(inner.take(length)),
            })
        }
    }
}

impl<'a, R> Read for CompressedRead<'a, R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.compression {
            Compression::None(ref mut inner) => inner.read(buf),
            Compression::Xz(ref mut inner) => inner.read(buf),
        }
    }
}
