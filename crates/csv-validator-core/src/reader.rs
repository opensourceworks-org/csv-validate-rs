use memchr::memchr;
use memmap2::Mmap;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Result};

pub struct OptimizedQuoteAwareReader {
    reader: BufReader<File>,
    buf: Vec<u8>,
}

impl OptimizedQuoteAwareReader {
    // todo: either derive or pass in the capacity as a parameter
    // same for the quote character
    pub fn open(path: &str, capacity: usize) -> Result<Self> {
        Ok(Self {
            reader: BufReader::with_capacity(capacity, File::open(path)?),
            buf: Vec::with_capacity(8192),
        })
    }

    pub fn next_logical_line<'a>(&mut self, line_buf: &'a mut Vec<u8>) -> Result<Option<&'a [u8]>> {
        line_buf.clear();
        let mut quote_count = 0;

        loop {
            self.buf.clear();
            let bytes_read = self.reader.read_until(b'\n', &mut self.buf)?;

            if bytes_read == 0 {
                if line_buf.is_empty() {
                    return Ok(None);
                } else {
                    return Ok(Some(line_buf));
                }
            }

            line_buf.extend_from_slice(&self.buf);

            // count quotes in the current line only at boundaries using simd bytecount
            quote_count += bytecount::count(&self.buf, b'"');

            // If quotes balanced explicitly, we have a complete logical line
            if quote_count % 2 == 0 {
                if line_buf.ends_with(&[b'\n']) {
                    line_buf.pop(); // remove trailing newline explicitly
                }
                return Ok(Some(line_buf));
            }
            // else continue reading explicitly
        }
    }
}

pub struct QuoteAwareBufferedReader {
    reader: BufReader<File>,
    buffer: Vec<u8>,
    position: usize,
    bytes_read: usize,
    eof: bool,
    in_quotes: bool,
}

impl QuoteAwareBufferedReader {
    pub fn open(path: &str, buffer_capacity: usize) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::with_capacity(buffer_capacity, file),
            buffer: vec![0; buffer_capacity],
            position: 0,
            bytes_read: 0,
            eof: false,
            in_quotes: false,
        })
    }

    // done: too slow, replace
    // Reads next logical line, accounting explicitly for quoted newlines
    pub fn next_logical_line<'a>(&mut self, line_buf: &'a mut Vec<u8>) -> Result<Option<&'a [u8]>> {
        line_buf.clear();
        loop {
            if self.position >= self.bytes_read {
                // Refill buffer explicitly
                self.bytes_read = self.reader.read(&mut self.buffer)?;
                self.position = 0;
                if self.bytes_read == 0 {
                    self.eof = true;
                    if !line_buf.is_empty() {
                        return Ok(Some(line_buf));
                    } else {
                        return Ok(None);
                    }
                }
            }

            let b = self.buffer[self.position];
            self.position += 1;

            match b {
                b'"' => {
                    self.in_quotes = !self.in_quotes; // explicitly toggle quote state
                    line_buf.push(b);
                }
                b'\n' => {
                    line_buf.push(b);
                    if !self.in_quotes {
                        if line_buf.ends_with(&[b'\n']) {
                            line_buf.pop(); // explicitly remove newline for consistency
                        }
                        return Ok(Some(line_buf));
                    }
                }
                _ => line_buf.push(b),
            }
        }
    }
}

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

/// done: too slow for sequential one-pass reads, replace
///
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

pub trait BufferedLineReader {
    fn next_line(&mut self, buf: &mut String) -> Result<bool>;
}

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

/// Naive buffered reader for files
impl BufferedLineReader for FileBufferedReader {
    fn next_line(&mut self, buf: &mut String) -> Result<bool> {
        buf.clear(); // explicitly clear buffer before reuse
        let bytes = self.reader.read_line(buf)?;
        Ok(bytes != 0) // EOF if 0 bytes read
    }
}

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
