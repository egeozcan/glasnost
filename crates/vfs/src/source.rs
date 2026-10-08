//! Byte sources: the files the user picked, read lazily by range.
//!
//! Reads are asynchronous so the same mounting code runs natively (where
//! [`FileSource`] answers immediately) and in the browser (where a `File`
//! is read with `Blob.slice` and a promise).

use std::future::Future;
use std::io;
use std::pin::Pin;

/// Future returned by [`ByteSource::read_at`].
pub type ReadFuture<'a> = Pin<Box<dyn Future<Output = io::Result<Vec<u8>>> + 'a>>;

/// A file that can be read by byte range.
pub trait ByteSource {
    /// Length in bytes.
    fn len(&self) -> u64;

    /// Whether the source is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads `len` bytes at `offset`. Fails if the range is not wholly
    /// inside the source.
    fn read_at(&self, offset: u64, len: usize) -> ReadFuture<'_>;
}

fn out_of_range() -> io::Error {
    io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "read past the end of the source",
    )
}

/// Whether `[offset, offset + len)` lies inside a source of `total` bytes.
pub(crate) fn in_range(total: u64, offset: u64, len: usize) -> bool {
    offset
        .checked_add(len as u64)
        .is_some_and(|end| end <= total)
}

/// A source held in memory (tests, and small files on the web).
#[derive(Clone, Debug, Default)]
pub struct MemSource(pub Vec<u8>);

impl ByteSource for MemSource {
    fn len(&self) -> u64 {
        self.0.len() as u64
    }

    fn read_at(&self, offset: u64, len: usize) -> ReadFuture<'_> {
        let result = if in_range(self.len(), offset, len) {
            // In range, so the conversion and slice cannot fail.
            let start = offset as usize;
            Ok(self.0[start..start + len].to_vec())
        } else {
            Err(out_of_range())
        };
        Box::pin(std::future::ready(result))
    }
}

/// A file on the local file system (native builds).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct FileSource {
    file: std::sync::Mutex<std::fs::File>,
    len: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileSource {
    /// Opens `path` for reading.
    pub fn open(path: &std::path::Path) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self {
            file: std::sync::Mutex::new(file),
            len,
        })
    }

    fn read_sync(&self, offset: u64, len: usize) -> io::Result<Vec<u8>> {
        use std::io::{Read, Seek, SeekFrom};
        if !in_range(self.len, offset, len) {
            return Err(out_of_range());
        }
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("file lock poisoned"))?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; len];
        file.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ByteSource for FileSource {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_at(&self, offset: u64, len: usize) -> ReadFuture<'_> {
        Box::pin(std::future::ready(self.read_sync(offset, len)))
    }
}

/// A `File` picked in the browser, read with `Blob.slice`.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct WebFileSource {
    file: web_sys::File,
    len: u64,
}

#[cfg(target_arch = "wasm32")]
impl WebFileSource {
    /// Wraps a picked file.
    pub fn new(file: web_sys::File) -> Self {
        // File sizes are integral and far below 2^53, so this is exact.
        let len = file.size() as u64;
        Self { file, len }
    }
}

#[cfg(target_arch = "wasm32")]
impl ByteSource for WebFileSource {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_at(&self, offset: u64, len: usize) -> ReadFuture<'_> {
        Box::pin(async move {
            if !in_range(self.len, offset, len) {
                return Err(out_of_range());
            }
            let js = |e: wasm_bindgen::JsValue| io::Error::other(format!("{e:?}"));
            // Offsets below 2^53 convert to f64 exactly.
            let start = offset as f64;
            let end = (offset + len as u64) as f64;
            let blob = self.file.slice_with_f64_and_f64(start, end).map_err(js)?;
            let buffer = wasm_bindgen_futures::JsFuture::from(blob.array_buffer())
                .await
                .map_err(js)?;
            let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
            if bytes.len() == len {
                Ok(bytes)
            } else {
                Err(out_of_range())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_source_checks_ranges() {
        let s = MemSource(vec![1, 2, 3, 4]);
        assert_eq!(pollster::block_on(s.read_at(1, 2)).unwrap(), vec![2, 3]);
        assert!(pollster::block_on(s.read_at(3, 2)).is_err());
        assert!(pollster::block_on(s.read_at(u64::MAX, 2)).is_err());
    }
}
