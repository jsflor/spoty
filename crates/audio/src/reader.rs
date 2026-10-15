use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, sync_channel};

/// Trait representing an audio stream reader that can be decoded by rodio/symphonia.
pub trait AudioStreamReader: Read + Seek + Send + Sync + 'static {}
impl<T: Read + Seek + Send + Sync + 'static> AudioStreamReader for T {}

/// A memory-bounded streaming bridge that consumes async or chunked audio data
/// without buffering entire files in memory, keeping RAM strictly <50MB.
pub struct BoundedStreamReader {
    rx: Mutex<Receiver<io::Result<Vec<u8>>>>,
    current_chunk: Vec<u8>,
    chunk_pos: usize,
    pos: u64,
    header_cache: Vec<u8>,
    header_cache_limit: usize,
}

impl BoundedStreamReader {
    pub const DEFAULT_MAX_HEADER_CACHE: usize = 128 * 1024; // 128 KB for codec headers
    pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024; // 64 KB per chunk

    pub fn new(rx: Receiver<io::Result<Vec<u8>>>) -> Self {
        Self {
            rx: Mutex::new(rx),
            current_chunk: Vec::new(),
            chunk_pos: 0,
            pos: 0,
            header_cache: Vec::new(),
            header_cache_limit: Self::DEFAULT_MAX_HEADER_CACHE,
        }
    }

    /// Spawns a background task reading from an `AsyncRead` into a bounded channel.
    pub fn from_async_read<R>(
        mut async_reader: R,
        tokio_rt: &tokio::runtime::Runtime,
    ) -> Box<dyn AudioStreamReader>
    where
        R: tokio::io::AsyncRead + Send + Unpin + 'static,
    {
        use tokio::io::AsyncReadExt;

        let (tx, rx) = sync_channel(4); // Bound queue to at most 4 chunks (256 KB RAM)

        tokio_rt.spawn(async move {
            let mut buf = vec![0u8; Self::DEFAULT_CHUNK_SIZE];
            loop {
                match async_reader.read(&mut buf).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let chunk = buf[..n].to_vec();
                        if tx.send(Ok(chunk)).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        break;
                    }
                }
            }
        });

        Box::new(Self::new(rx))
    }

    /// Helper to create an `AudioStreamReader` from in-memory bytes (useful for tests and demo tracks).
    pub fn from_bytes(bytes: Vec<u8>) -> Box<dyn AudioStreamReader> {
        Box::new(Cursor::new(bytes))
    }
}

impl Read for BoundedStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total_read = 0;
        while total_read < buf.len() {
            // If serving from header cache
            if self.pos < self.header_cache.len() as u64 {
                let cache_pos = self.pos as usize;
                let available = self.header_cache.len() - cache_pos;
                let to_read = available.min(buf.len() - total_read);
                buf[total_read..total_read + to_read]
                    .copy_from_slice(&self.header_cache[cache_pos..cache_pos + to_read]);
                self.pos += to_read as u64;
                total_read += to_read;
                continue;
            }

            // If current chunk exhausted, fetch next chunk
            if self.chunk_pos >= self.current_chunk.len() {
                let chunk_res = {
                    let rx_lock = self
                        .rx
                        .lock()
                        .map_err(|_| io::Error::other("BoundedStreamReader mutex poisoned"))?;
                    rx_lock.recv()
                };

                match chunk_res {
                    Ok(Ok(chunk)) => {
                        self.current_chunk = chunk;
                        self.chunk_pos = 0;
                    }
                    Ok(Err(e)) => {
                        if total_read > 0 {
                            return Ok(total_read);
                        }
                        return Err(e);
                    }
                    Err(_) => {
                        // EOF
                        break;
                    }
                }
            }

            let available = self.current_chunk.len() - self.chunk_pos;
            if available == 0 {
                break;
            }
            let to_read = available.min(buf.len() - total_read);
            buf[total_read..total_read + to_read]
                .copy_from_slice(&self.current_chunk[self.chunk_pos..self.chunk_pos + to_read]);

            if self.header_cache.len() < self.header_cache_limit {
                let cache_remaining = self.header_cache_limit - self.header_cache.len();
                let cache_add = to_read.min(cache_remaining);
                self.header_cache.extend_from_slice(
                    &self.current_chunk[self.chunk_pos..self.chunk_pos + cache_add],
                );
            }

            self.chunk_pos += to_read;
            self.pos += to_read as u64;
            total_read += to_read;
        }

        Ok(total_read)
    }
}

impl Seek for BoundedStreamReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(target) => {
                if target <= self.header_cache.len() as u64 {
                    self.pos = target;
                    Ok(self.pos)
                } else if target == self.pos {
                    Ok(self.pos)
                } else if target > self.pos {
                    let to_skip = (target - self.pos) as usize;
                    let mut discard = vec![0u8; to_skip.min(8192)];
                    let mut skipped = 0;
                    while skipped < to_skip {
                        let step = (to_skip - skipped).min(discard.len());
                        let n = self.read(&mut discard[..step])?;
                        if n == 0 {
                            break;
                        }
                        skipped += n;
                    }
                    Ok(self.pos)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Backward seek past cached header boundary is not supported on streaming input",
                    ))
                }
            }
            SeekFrom::Current(offset) => {
                let target = (self.pos as i64 + offset).max(0) as u64;
                self.seek(SeekFrom::Start(target))
            }
            SeekFrom::End(_) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "SeekFrom::End is not supported on streaming audio reader",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_stream_reader_sequential_read() {
        let (tx, rx) = sync_channel(2);
        tx.send(Ok(vec![1, 2, 3, 4])).unwrap();
        tx.send(Ok(vec![5, 6, 7, 8])).unwrap();
        drop(tx);

        let mut reader = BoundedStreamReader::new(rx);
        let mut buf = [0u8; 3];

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&buf[..n], &[1, 2, 3]);

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&buf[..n], &[4, 5, 6]);

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&buf[..n], &[7, 8]);

        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 0); // EOF
    }

    #[test]
    fn test_bounded_stream_reader_header_seek() {
        let (tx, rx) = sync_channel(2);
        tx.send(Ok(vec![10, 20, 30, 40, 50])).unwrap();
        drop(tx);

        let mut reader = BoundedStreamReader::new(rx);
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, &[10, 20]);

        // Rewind to 0 within header cache
        let new_pos = reader.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(new_pos, 0);

        let mut full_buf = [0u8; 5];
        reader.read_exact(&mut full_buf).unwrap();
        assert_eq!(&full_buf, &[10, 20, 30, 40, 50]);
    }
}
