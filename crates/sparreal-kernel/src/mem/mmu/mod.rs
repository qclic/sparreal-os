use core::{alloc::Layout, ffi::CStr, ops::Range, ptr::NonNull};

use buddy_system_allocator::Heap;
use page_table_generic::err::PagingError;
pub use page_table_generic::*;

mod paging;

pub use paging::iomap;

use crate::{
    globals::global_val,
    io::print::{
        early_dbg, early_dbg_fmt, early_dbg_hex, early_dbg_hexln, early_dbg_range, early_dbgln,
    },
    platform_if::{MMUImpl, PlatformImpl},
};

use paging::PageTableRef;
pub use paging::init_table;

pub use super::addr2::PhysRange;
use super::{Align, va_offset};

pub use arrayvec::ArrayVec;

struct PageHeap(Heap<32>);

impl page_table_generic::Access for PageHeap {
    fn va_offset(&self) -> usize {
        0
    }

    unsafe fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        self.0.alloc(layout).ok()
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        self.0.dealloc(ptr, layout);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RsvRegion {
    pub range: PhysRange,
    pub name: *const u8,
    pub access: AccessSetting,
    pub cache: CacheSetting,
    pub kind: RsvRegionKind,
}

impl RsvRegion {
    pub fn new(
        range: PhysRange,
        name: &'static CStr,
        access: AccessSetting,
        cache: CacheSetting,
        kind: RsvRegionKind,
    ) -> Self {
        Self {
            range,
            name: name.as_ptr(),
            access,
            cache,
            kind,
        }
    }

    pub fn name(&self) -> &'static str {
        unsafe { CStr::from_ptr(self.name).to_str().unwrap() }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum RsvRegionKind {
    Image,
    Stack,
    Other,
}

pub struct BootMemoryRegion {
    pub name: &'static str,
    pub range: Range<usize>,
    pub access: AccessSetting,
    pub cache: CacheSetting,
}
pub fn new_boot_table() -> Result<usize, &'static str> {

    let mut access = PageHeap(Heap::empty());

    let stack_region = MMUImpl::rsv_regions()
        .into_iter()
        .find(|&a| matches!(a.kind, RsvRegionKind::Stack))
        .unwrap();

    // 临时用栈底储存页表项
    let tmp_pt = stack_region.range.start.raw();

    unsafe { access.0.init(tmp_pt, 1024 * 1024) };

    let mut table =
        PageTableRef::create_empty(&mut access).map_err(|_| "page table allocator no memory")?;

    for region in MMUImpl::rsv_regions() {
        early_dbgln(region.name());
    }



    let table_addr = table.paddr();

    early_dbg("Table: ");
    early_dbg_hexln(table_addr as _);

    Ok(table_addr)
}

fn map_region(
    table: &mut PageTableRef<'_>,
    va_offset: usize,
    region: &BootMemoryRegion,
    access: &mut PageHeap,
) {
    let addr = region.range.start;
    let size = region.range.end - region.range.start;

    // let addr = align_down_1g(addr);
    // let size = align_up_1g(size);
    let vaddr = addr + va_offset;

    early_dbg("map region: [");
    early_dbg(region.name);
    early_dbg("] [");
    early_dbg_hex(vaddr as _);
    early_dbg(", ");
    early_dbg_hex((vaddr + size) as _);
    early_dbg(") -> [");
    early_dbg_hex(addr as _);
    early_dbg(", ");
    early_dbg_hex((addr + size) as _);
    early_dbgln(")");

    unsafe {
        if let Err(e) = table.map_region(
            MapConfig::new(addr as _, addr, region.access, region.cache),
            size,
            true,
            access,
        ) {
            early_handle_err(e);
        }

        if let Err(e) = table.map_region(
            MapConfig::new(vaddr as _, addr, region.access, region.cache),
            size,
            true,
            access,
        ) {
            early_handle_err(e);
        }
    }
}

fn early_handle_err(e: PagingError) {
    match e {
        PagingError::NoMemory => early_dbgln("no memory"),
        PagingError::NotAligned(e) => {
            early_dbg(e);
            early_dbgln(" not aligned");
        }
        PagingError::NotMapped => early_dbgln("not mapped"),
        PagingError::AlreadyMapped => {}
    }
    panic!()
}

pub fn set_kernel_table(addr: usize) {
    MMUImpl::set_kernel_table(addr);
}

pub fn set_user_table(addr: usize) {
    MMUImpl::set_user_table(addr);
}
pub fn get_user_table() -> usize {
    MMUImpl::get_user_table()
}

#[allow(unused)]
pub(crate) fn flush_tlb(addr: *const u8) {
    unsafe { MMUImpl::flush_tlb(addr) };
}
pub fn flush_tlb_all() {
    MMUImpl::flush_tlb_all();
}
pub fn page_size() -> usize {
    MMUImpl::page_size()
}
pub fn table_level() -> usize {
    MMUImpl::table_level()
}
