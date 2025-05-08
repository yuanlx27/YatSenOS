use alloc::format;
//use elf::map_range;
use x86_64::{
    structures::paging::*,
    VirtAddr,
};

use crate::{humanized_size, memory::*};

pub mod stack;
pub use stack::*;

use super::PageTableContext;

use xmas_elf::ElfFile;

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();

        self
    }

    //pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {
    //    // DONE: calculate the stack for pid
    //    let stack_top_addr = STACK_INIT_TOP - (pid.0 as u64 - 1) * STACK_MAX_SIZE;
    //    let frame_allocator = &mut *get_frame_alloc_for_sure();

    //    map_range(stack_top_addr, STACK_DEF_PAGE, &mut self.page_table.mapper(), frame_allocator).unwrap();

    //    let stack_top_vaddr = VirtAddr::new(stack_top_addr);

    //    self.stack = Stack::new(
    //        Page::containing_address(stack_top_vaddr),
    //        STACK_DEF_PAGE,
    //    );

    //    stack_top_vaddr
    //}

    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.init(mapper, alloc);
        self.load_elf_code(elf, mapper, alloc);
    }
    fn load_elf_code(&mut self, elf: &ElfFile, mapper: MapperRef, alloc: FrameAllocatorRef) {
        elf::load_elf(elf, *PHYSICAL_OFFSET.get().unwrap(), mapper, alloc, true).unwrap();
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage()
    }

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let new_page_table = self.page_table.fork();

        let mapper = &mut new_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: new_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
        }
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
