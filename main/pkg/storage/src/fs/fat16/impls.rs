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

    // DONE: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //       - read the FAT and get next cluster
    //       - traverse the cluster chain and read the data
    //       - parse the path
    //       - open the root directory
    //       - ...
    //       - finally, implement the FileSystem trait for Fat16 with `self.handle`
    pub fn next_cluster(&self, cluster: &Cluster) -> FsResult<Cluster> {
        let fat_offset = cluster.0 as usize * 2;
        let block_size = Block512::size();
        let sector = self.fat_start + fat_offset / block_size;
        let offset = fat_offset % block_size;

        let mut block = Block::default();
        self.inner.read_block(sector, &mut block)?;

        let fat_entry = u16::from_le_bytes(block[offset..offset + 2].try_into().unwrap_or([ 0; 2 ]));
        match fat_entry {
            0xFFF7 => Err(FsError::BadCluster),
            0xFFF8 => Err(FsError::EndOfFile),
            f => Ok(Cluster(f as u32)),
        }
    }

    pub fn iterate_dir<F>(&self, dir: &directory::Directory, mut func: F) -> FsResult
    where
        F: FnMut(&DirEntry),
    {
        if let Some(entry) = &dir.entry {
            trace!("Iterating directory: {}", entry.filename());
        }

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        trace!("Directory size: {}", dir_size);

        let mut block = Block::default();
        let block_size = Block512::size();
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                self.inner.read_block(sector, &mut block).unwrap();
                for entry in 0..block_size / DirEntry::LEN {
                    let start = entry * DirEntry::LEN;
                    let end = (entry + 1) * DirEntry::LEN;

                    let dir_entry = DirEntry::parse(&block[start..end])?;

                    if dir_entry.is_eod() {
                        return Ok(());
                    } else if dir_entry.is_valid() && !dir_entry.is_long_name() {
                        func(&dir_entry);
                    }
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(&cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ok(())
    }

    fn find_entry_in_sector(&self, name: &ShortFileName, sector: usize) -> FsResult<DirEntry> {
        let mut block = Block::default();
        self.inner.read_block(sector, &mut block)?;

        for entry in 0..Block512::size() / DirEntry::LEN {
            let dir_entry = DirEntry::parse(&block[entry * DirEntry::LEN..(entry + 1) * DirEntry::LEN])
                .map_err(|_| FsError::InvalidOperation)?;

            if dir_entry.is_eod() {
                return Err(FsError::FileNotFound);
            } else if dir_entry.filename.matches(name) {
                return Ok(dir_entry);
            }
        }

        Err(FsError::NotInSector)
    }
    fn find_entry_in_dir(&self, name: &str, dir: &Directory) -> FsResult<DirEntry> {
        let name = ShortFileName::parse(name)?;
        let size = match dir.cluster {
            Cluster::ROOT_DIR => self.bpb.root_entries_count() as usize * DirEntry::LEN,
            _ => self.bpb.sectors_per_cluster() as usize * Block512::size(),
        };

        let mut current_cluster = Some(dir.cluster);
        while let Some(cluster) = current_cluster {
            let current_sector = self.cluster_to_sector(&cluster);
            for sector in current_sector..current_sector + size {
                if let Ok(entry) = self.find_entry_in_sector(&name, sector) {
                    return Ok(entry);
                }
            }

            if cluster == Cluster::ROOT_DIR {
                break;
            }

            current_cluster = self.next_cluster(&cluster).ok();
        }

        Err(FsError::FileNotFound)
    }

    fn get_dir(&self, path: &str) -> FsResult<Directory> {
        let mut path = path.split('/');
        let mut current = Directory::root();

        while let Some(dir) = path.next() {
            if dir.is_empty() {
                continue;
            }

            let entry = self.find_entry_in_dir(dir, &current)?;
            if entry.is_directory() {
                current = Directory::from_entry(entry);
            } else if path.next().is_some() {
                return Err(FsError::NotADirectory);
            } else {
                break;
            }
        }

        Ok(current)
    }
    fn get_entry(&self, path: &str) -> FsResult<DirEntry> {
        let dir = self.get_dir(path)?;
        let name = path.rsplit('/').next().unwrap_or("");

        self.find_entry_in_dir(name, &dir)
    }
}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> FsResult<Box<dyn Iterator<Item = Metadata> + Send>> {
        // DONE: read dir and return an iterator for all entries
        let dir = self.handle.get_dir(path)?;

        let mut entries = Vec::new();
        self.handle.iterate_dir(&dir, |entry| {
            entries.push(entry.as_meta());
        })?;

        Ok(Box::new(entries.into_iter()))
    }

    fn open_file(&self, path: &str) -> FsResult<FileHandle> {
        // DONE: open file and return a file handle
        let entry = self.handle.get_entry(path)?;
        let handle = self.handle.clone();

        if entry.is_directory() {
            return Err(FsError::NotAFile);
        }

        Ok(FileHandle::new(entry.as_meta(), Box::new(File::new(handle, entry))))
    }

    fn metadata(&self, path: &str) -> FsResult<Metadata> {
        // DONE: read metadata of the file / dir
        Ok(self.handle.get_entry(path).unwrap().as_meta())
    }

    fn exists(&self, path: &str) -> FsResult<bool> {
        // DONE: check if the file / dir exists
        Ok(self.handle.get_entry(path).is_ok())
    }
}
