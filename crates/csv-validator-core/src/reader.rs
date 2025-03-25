use memchr::memchr;
use std::fs::File;
use std::io::{Cursor, Result, BufReader, BufRead};
use memmap2::Mmap;


pub struct FastBufferedReader {
    pub reader: BufReader<File>,
    pub buffer: Vec<u8>,
}

impl FastBufferedReader {
    pub fn open(path: &str, capacity: usize) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::with_capacity(capacity, file),
            buffer: Vec::with_capacity(1024), // explicitly reused buffer
        })
    }

    pub fn next_line(&mut self) -> Result<Option<&[u8]>> {
        self.buffer.clear();
        let bytes_read = self.reader.read_until(b'\n', &mut self.buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        if self.buffer.ends_with(&[b'\n']) {
            self.buffer.pop();
        }
        Ok(Some(&self.buffer))
    }
}


/// Buffered reader using memory mapping explicitly
pub struct MmapBufferedReader {
    pub mmap: Mmap,
    pub position: usize,
}

impl MmapBufferedReader {
    pub fn open(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap, position: 0 })
    }
}

impl BufferedLineReader for MmapBufferedReader {
    fn next_line(&mut self, buf: &mut String) -> Result<bool> {
        buf.clear();

        if self.position >= self.mmap.len() {
            return Ok(false); // EOF explicitly reached
        }

        let slice = &self.mmap[self.position..];

        match memchr(b'\n', slice) {
            Some(newline_pos) => {
                let line = &slice[..newline_pos];
                *buf = String::from_utf8_lossy(line).to_string();
                self.position += newline_pos + 1;
            }
            None => {
                // Last line without newline
                *buf = String::from_utf8_lossy(slice).to_string();
                self.position = self.mmap.len();
            }
        }

        Ok(true)
    }
}


/// Trait for explicitly abstracting buffered input sources
pub trait BufferedLineReader {
    /// Reads the next line into the provided String buffer.
    /// Returns Ok(true) if a line was read, Ok(false) if EOF reached.
    fn next_line(&mut self, buf: &mut String) -> Result<bool>;
}


/// Explicit buffered reader from a file
pub struct FileBufferedReader {
    reader: BufReader<File>,
}

impl FileBufferedReader {
    pub fn open(path: &str, buffer_capacity: usize) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::with_capacity(buffer_capacity, file),
        })
    }
}

impl BufferedLineReader for FileBufferedReader {
    fn next_line(&mut self, buf: &mut String) -> Result<bool> {
        buf.clear();  // explicitly clear buffer before reuse
        let bytes = self.reader.read_line(buf)?;
        Ok(bytes != 0)  // EOF if 0 bytes read
    }
}



/// Explicit buffered reader from an in-memory buffer
pub struct MemoryBufferedReader<'a> {
    reader: BufReader<Cursor<&'a [u8]>>,
}

impl<'a> MemoryBufferedReader<'a> {
    pub fn new(buffer: &'a [u8], buffer_capacity: usize) -> Self {
        Self {
            reader: BufReader::with_capacity(buffer_capacity, Cursor::new(buffer)),
        }
    }
}

impl<'a> BufferedLineReader for MemoryBufferedReader<'a> {
    fn next_line(&mut self, buf: &mut String) -> Result<bool> {
        buf.clear();
        let bytes = self.reader.read_line(buf)?;
        Ok(bytes != 0)
    }
}
