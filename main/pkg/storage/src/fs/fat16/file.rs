//! File
//!
//! reference: <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>

use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file
    offset: usize,
    /// The current cluster of this file
    current_cluster: Cluster,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            current_cluster: entry.cluster,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {
        // DONE: read file content from disk
        // CAUTION: file length / buffer size / offset
        //       - `self.offset` is the current offset in the file in bytes
        //       - use `self.handle` to read the blocks
        //       - use `self.entry` to get the file's cluster
        //       - use `self.handle.cluster_to_sector` to convert cluster to sector
        //       - update `self.offset` after reading
        //       - update `self.cluster` with FAT if necessary
        let length = self.length();

        if self.offset >= length {
            return Ok(0); // EOF
        }

        let bytes_per_cluster = {
            let sectors_per_cluster = self.handle.bpb.sectors_per_cluster() as usize;
            let bytes_per_sector = self.handle.bpb.bytes_per_sector() as usize;
            sectors_per_cluster * bytes_per_sector
        };

        let mut block = Block::default();
        let mut bytes_read = 0;
        while bytes_read < buf.len() && self.offset < length {
            let current_sector = {
                let cluster_sector = self.handle.cluster_to_sector(&self.current_cluster);
                let cluster_offset = (self.offset % bytes_per_cluster) / BLOCK_SIZE;
                cluster_sector + cluster_offset
            };

            self.handle.inner.read_block(current_sector, &mut block)?;

            let block_offset = self.offset % BLOCK_SIZE;
            let block_remain = BLOCK_SIZE - block_offset;

            let to_read = block_remain.min(buf.len() - bytes_read).min(length - self.offset);

            buf[bytes_read..bytes_read + to_read]
                .copy_from_slice(&block[block_offset..block_offset + to_read]);

            bytes_read += to_read;
            self.offset += to_read;

            if self.offset % bytes_per_cluster == 0 {
                if let Ok(next_cluster) = self.handle.next_cluster(&self.current_cluster) {
                    self.current_cluster = next_cluster;
                } else {
                    break;
                }
            }
        }

        Ok(bytes_read)
    }
}

// NOTE: `Seek` trait is not required for this lab
impl Seek for File {
    fn seek(&mut self, _pos: SeekFrom) -> FsResult<usize> {
        unimplemented!()
    }
}

// NOTE: `Write` trait is not required for this lab
impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> FsResult<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> FsResult {
        unimplemented!()
    }
}
