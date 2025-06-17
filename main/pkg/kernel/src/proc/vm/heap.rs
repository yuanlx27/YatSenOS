use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;
use x86_64::{
    structures::paging::{mapper::UnmapError, Page},
    VirtAddr,
};

use super::{FrameAllocatorRef, MapperRef};

// user process runtime heap
// 0x100000000 bytes -> 4GiB
// from 0x0000_2000_0000_0000 to 0x0000_2000_ffff_fff8
pub const HEAP_START: u64 = 0x2000_0000_0000;
pub const HEAP_PAGES: u64 = 0x100000;
pub const HEAP_SIZE: u64 = HEAP_PAGES * crate::memory::PAGE_SIZE;
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE - 8;

/// User process runtime heap
///
/// always page aligned, the range is [base, end)
pub struct Heap {
    /// the base address of the heap
    ///
    /// immutable after initialization
    base: VirtAddr,

    /// the current end address of the heap
    ///
    /// use atomic to allow multiple threads to access the heap
    end: Arc<AtomicU64>,
}

impl Heap {
    pub fn empty() -> Self {
        Self {
            base: VirtAddr::new(HEAP_START),
            end: Arc::new(AtomicU64::new(HEAP_START)),
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            base: self.base,
            end: self.end.clone(),
        }
    }

    pub fn brk(
        &self,
        new_end: Option<VirtAddr>,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> Option<VirtAddr> {
        // DONE: if new_end is None, return the current end address
        let Some(new_end) = new_end else {
            return Some(VirtAddr::new(self.end.load(Ordering::Relaxed)));
        };

        // DONE: check if the new_end is valid (in range [base, base + HEAP_SIZE])
        if new_end < self.base || self.base + HEAP_SIZE < new_end {
            error!("Heap::brk: new end is out of range");
            return None;
        }

        // DONE: calculate the difference between the current end and the new end
        let cur_end = self.end.load(Ordering::Acquire);

        // Calculate pages.
        let mut cur_end_page = Page::containing_address(VirtAddr::new(cur_end));
        if cur_end != self.base.as_u64() {
            cur_end_page += 1;
        }
        let mut new_end_page = Page::containing_address(new_end);
        if new_end != self.base {
            new_end_page += 1;
        }

        // DONE: print the heap difference for debugging
        debug!("Heap end addr: {:#x} -> {:#x}", cur_end, new_end.as_u64());
        debug!(
            "Heap end page: {:#x} -> {:#x}",
            cur_end_page.start_address().as_u64(),
            new_end_page.start_address().as_u64()
        );

        // DONE: do the actual mapping or unmapping
        match new_end_page.cmp(&cur_end_page) {
            core::cmp::Ordering::Greater => {
                let range = Page::range_inclusive(cur_end_page, new_end_page - 1);
                elf::map_range(range, mapper, alloc, true).ok()?;
            },
            core::cmp::Ordering::Less => {
                let range = Page::range_inclusive(new_end_page, cur_end_page - 1);
                elf::unmap_range(range, mapper, alloc, true).ok()?;
            },
            core::cmp::Ordering::Equal => {},
        }

        // DONE: update the end address
        self.end.store(new_end.as_u64(), Ordering::Release);

        Some(new_end)
    }

    pub(super) fn clean_up(
        &self,
        mapper: MapperRef,
        dealloc: FrameAllocatorRef,
    ) -> Result<(), UnmapError> {
        if self.memory_usage() == 0 {
            return Ok(());
        }

        // DONE: load the current end address and **reset it to base** (use `swap`)
        let end_addr = self.end.swap(self.base.as_u64(), Ordering::Release);
        let start_page = Page::containing_address(self.base);
        let end_page = Page::containing_address(VirtAddr::new(end_addr));
        let range = Page::range_inclusive(start_page, end_page);

        // DONE: unmap the heap pages
        elf::unmap_range(range, mapper, dealloc, true)?;

        Ok(())
    }

    pub fn memory_usage(&self) -> u64 {
        self.end.load(Ordering::Relaxed) - self.base.as_u64()
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("base", &format_args!("{:#x}", self.base.as_u64()))
            .field(
                "end",
                &format_args!("{:#x}", self.end.load(Ordering::Relaxed)),
            )
            .finish()
    }
}
