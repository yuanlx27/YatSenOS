use super::*;

impl Fat16Impl {
    pub fn new(inner: impl BlockDevice<Block512>) -> Self {
        let mut block = Block::default();
        let block_size = Block512::size();

        inner.read_block(0, &mut block).unwrap();
        let bpb = Fat16Bpb::new(block.as_ref()).unwrap();

        trace!("Loading Fat16 Volume: {:#?}", bpb);

        // HINT: FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let fat_start = bpb.reserved_sector_count() as usize;
        let root_dir_size = (bpb.root_entries_count() as usize * DirEntry::LEN).div_ceil(block_size);
        let first_root_dir_sector = bpb.reserved_sector_count() as usize + bpb.fat_count() as usize * bpb.sectors_per_fat() as usize;
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            bpb,
            inner: Box::new(inner),
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }

    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // DONE: calculate the first sector of the cluster
                // HINT: FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                let first_sector_of_cluster = (c - 2) * self.bpb.sectors_per_cluster() as u32;
                first_sector_of_cluster as usize + self.first_data_sector
            }
        }
    }

    // FIXME: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //      - read the FAT and get next cluster
    //      - traverse the cluster chain and read the data
    //      - parse the path
    //      - open the root directory
    //      - ...
    //      - finally, implement the FileSystem trait for Fat16 with `self.handle`
    fn next_cluster(&self, cluster: Cluster) -> FsResult<Cluster> {
        let fat_offset = cluster.0 as usize * 2;
        let block_size = Block512::size();
        let sector_to_read = self.fat_start + fat_offset / block_size;
        let offset_in_block = fat_offset % block_size;

        let mut block = Block::default();
        self.inner.read_block(sector_to_read, &mut block)?;

        let fat_entry = u16::from_le_bytes(block[offset_in_block..offset_in_block + 2].try_into().unwrap_or([ 0; 2 ]));
        match fat_entry {
            0xFFF7 => Err(FsError::BadCluster),
            0xFFF8 => Err(FsError::EndOfFile),
            f => Ok(Cluster(f as u32)),
        }
    }
}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> FsResult<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read dir and return an iterator for all entries
        todo!()
    }

    fn open_file(&self, path: &str) -> FsResult<FileHandle> {
        // FIXME: open file and return a file handle
        todo!()
    }

    fn metadata(&self, path: &str) -> FsResult<Metadata> {
        // FIXME: read metadata of the file / dir
        todo!()
    }

    fn exists(&self, path: &str) -> FsResult<bool> {
        // FIXME: check if the file / dir exists
        todo!()
    }
}
