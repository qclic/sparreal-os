use core::{
    alloc::Layout,
    ffi::CStr,
    ops::Range,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use super::{Phys, PhysAddr, PhysCRange, STACK_BOTTOM, Virt, once::OnceStatic};
pub use arrayvec::ArrayVec;
use buddy_system_allocator::Heap;
use page_table_generic::err::PagingError;
pub use page_table_generic::*;

use crate::{
    globals::{self, cpu_inited, global_val},
    io::print::*,
    platform,
    platform_if::{MMUImpl, PlatformImpl},
};

mod paging;

pub use paging::init_table;
pub use paging::iomap;

pub const LINER_OFFSET: usize = 0xffff_f000_0000_0000;
static TEXT_OFFSET: AtomicUsize = AtomicUsize::new(0);
static IS_MMU_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_mmu_enabled() {
    IS_MMU_ENABLED.store(true, Ordering::SeqCst);
}

pub fn is_mmu_enabled() -> bool {
    IS_MMU_ENABLED.load(Ordering::Relaxed)
}

pub fn set_text_va_offset(offset: usize) {
    TEXT_OFFSET.store(offset, Ordering::SeqCst);
}

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
    pub range: PhysCRange,
    pub name: *const u8,
    pub access: AccessSetting,
    pub cache: CacheSetting,
    pub kind: RegionKind,
}

impl RsvRegion {
    pub fn new(
        range: Range<PhysAddr>,
        name: &'static CStr,
        access: AccessSetting,
        cache: CacheSetting,
        kind: RegionKind,
    ) -> Self {
        Self {
            range: range.into(),
            name: name.as_ptr() as _,
            access,
            cache,
            kind,
        }
    }

    pub fn new_with_len(
        start: PhysAddr,
        len: usize,
        name: &'static CStr,
        access: AccessSetting,
        cache: CacheSetting,
        kind: RegionKind,
    ) -> Self {
        Self::new(start..start + len, name, access, cache, kind)
    }

    pub fn name(&self) -> &'static str {
        unsafe { CStr::from_ptr(self.name as _).to_str().unwrap() }
    }

    pub fn va_offset(&self) -> usize {
        match self.kind {
            RegionKind::Stack => {
                if cpu_inited() {
                    self.kind.va_offset()
                } else {
                    STACK_BOTTOM - self.range.start.raw()
                }
            }
            _ => self.kind.va_offset(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum RegionKind {
    KImage,
    Stack,
    Other,
}

impl RegionKind {
    pub fn va_offset(&self) -> usize {
        match self {
            RegionKind::KImage => TEXT_OFFSET.load(Ordering::Relaxed),
            RegionKind::Stack => STACK_BOTTOM - globals::cpu_global().stack.start.raw(),
            RegionKind::Other => LINER_OFFSET,
        }
    }
}

impl<T> From<Virt<T>> for Phys<T> {
    fn from(value: Virt<T>) -> Self {
        let v = value.raw();
        todo!()
    }
}
const MB: usize = 1024 * 1024;
pub fn new_boot_table() -> Result<usize, &'static str> {
    let mut access = PageHeap(Heap::empty());

    let tmp_end = global_val().main_memory.end;
    let tmp_size = tmp_end - global_val().main_memory.start.align_up(MB);
    let tmp_pt = (global_val().main_memory.end - tmp_size / 2).raw();

    early_dbg_range("page table allocator", tmp_pt..tmp_end.raw());
    unsafe { access.0.add_to_heap(tmp_pt, tmp_end.raw()) };

    let mut table =
        PageTableRef::create_empty(&mut access).map_err(|_| "page table allocator no memory")?;

    for memory in platform::phys_memorys() {
        let region = RsvRegion::new(
            memory,
            c"memory",
            AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
            CacheSetting::Normal,
            RegionKind::Other,
        );
        map_region(&mut table, 0, &region, &mut access);
    }

    for region in MMUImpl::rsv_regions() {
        map_region(&mut table, region.va_offset(), &region, &mut access);
    }

    let main_memory = RsvRegion::new(
        global_val().main_memory.clone(),
        c"main memory",
        AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
        CacheSetting::Normal,
        RegionKind::Other,
    );

    map_region(
        &mut table,
        main_memory.va_offset(),
        &main_memory,
        &mut access,
    );

    let table_addr = table.paddr();

    early_dbg("Table: ");
    early_dbg_hexln(table_addr as _);

    Ok(table_addr)
}

fn map_region(
    table: &mut paging::PageTableRef<'_>,
    va_offset: usize,
    region: &RsvRegion,
    access: &mut PageHeap,
) {
    let addr = region.range.start;
    let size = region.range.end.raw() - region.range.start.raw();

    // let addr = align_down_1g(addr);
    // let size = align_up_1g(size);
    let vaddr = addr.raw() + va_offset;

    const NAME_LEN: usize = 12;

    let name_right = if region.name().len() < NAME_LEN {
        NAME_LEN - region.name().len()
    } else {
        0
    };

    early_dbg("map region [");
    early_dbg(region.name());
    for _ in 0..name_right {
        early_dbg(" ");
    }
    early_dbg("] [");
    early_dbg_hex(vaddr as _);
    early_dbg(", ");
    early_dbg_hex((vaddr + size) as _);
    early_dbg(") -> [");
    early_dbg_hex(addr.raw() as _);
    early_dbg(", ");
    early_dbg_hex((addr.raw() + size) as _);
    early_dbgln(")");

    unsafe {
        if let Err(e) = table.map_region(
            MapConfig::new(vaddr as _, addr.raw(), region.access, region.cache),
            size,
            true,
            access,
        ) {
            // early_handle_err(e);
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
