use arrayvec::ArrayVec;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use sparreal_kernel::mem::addr2::*;
use sparreal_kernel::mem::mmu::*;
pub use sparreal_kernel::mem::*;
use sparreal_kernel::platform_if::RsvRegion;

unsafe extern "C" {
    fn _stext();
    fn _etext();
    fn _srodata();
    fn _erodata();
    fn _sdata();
    fn _edata();
    fn _sbss();
    fn _ebss();
    fn _stack_bottom();
    fn _stack_top();
}

macro_rules! fn_ld_range {
    ($name:ident) => {
        pub fn $name() -> &'static [u8] {
            let start = concat_idents!(_s, $name) as *const u8 as usize;
            let end = concat_idents!(_e, $name) as *const u8 as usize;
            unsafe { &*slice_from_raw_parts(start as *mut u8, end - start) }
        }
    };
}

fn_ld_range!(text);
fn_ld_range!(rodata);
fn_ld_range!(data);
fn_ld_range!(bss);

pub fn stack_cpu0() -> &'static [u8] {
    let start = _stack_bottom as *const u8 as usize;
    let end = _stack_top as *const u8 as usize;
    unsafe { &*slice_from_raw_parts(start as *mut u8, end - start) }
}

pub fn clean_bss() {
    let start = _sbss as *const u8 as usize;
    let end = _ebss as *const u8 as usize;
    let bss = unsafe { &mut *slice_from_raw_parts_mut(start as *mut u8, end - start) };
    bss.fill(0);
}

fn slice_to_phys_range(data: &[u8]) -> PhysRange {
    let ptr_range = data.as_ptr_range();
    PhysRange {
        start: (ptr_range.start as usize).into(),
        end: (ptr_range.end as usize).into(),
    }
}

pub fn rsv_regions<const N: usize>() -> ArrayVec<RsvRegion, N> {
    let mut rsv_regions = ArrayVec::<RsvRegion, N>::new();
    rsv_regions.push(RsvRegion::new(
        slice_to_phys_range(text()),
        c".text",
        AccessSetting::Read | AccessSetting::Execute,
        CacheSetting::Normal,
        RsvRegionKind::Image,
    ));

    rsv_regions.push(RsvRegion::new(
        slice_to_phys_range(rodata()),
        c".rodata",
        AccessSetting::Read | AccessSetting::Execute,
        CacheSetting::Normal,
        RsvRegionKind::Image,
    ));

    rsv_regions.push(RsvRegion::new(
        slice_to_phys_range(data()),
        c".data",
        AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
        CacheSetting::Normal,
        RsvRegionKind::Image,
    ));

    rsv_regions.push(RsvRegion::new(
        slice_to_phys_range(bss()),
        c".bss",
        AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
        CacheSetting::Normal,
        RsvRegionKind::Image,
    ));

    rsv_regions.push(RsvRegion::new(
        slice_to_phys_range(stack_cpu0()),
        c".stack",
        AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
        CacheSetting::Normal,
        RsvRegionKind::Stack,
    ));

    rsv_regions
}
