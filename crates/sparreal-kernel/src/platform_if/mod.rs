use core::ffi::c_void;

pub use driver_interface::DriverRegisterListRef;
use page_table_generic::PTEGeneric;
use sparreal_macros::api_trait;

use crate::mem::KernelRegions;

#[api_trait]
pub trait Platform {
    fn kernel_regions() -> KernelRegions;
    fn kstack_size() -> usize;
    fn cpu_id() -> usize;
    fn cpu_context_size() -> usize;

    /// # Safety
    ///
    ///
    unsafe fn get_current_tcb_addr() -> *const u8;

    /// # Safety
    ///
    ///
    unsafe fn set_current_tcb_addr(addr: *const u8);

    /// # Safety
    ///
    ///
    unsafe fn cpu_context_init(ctx_ptr: *mut u8, pc: *const c_void, stack_top: *const u8);

    /// # Safety
    ///
    /// `ctx_ptr` 是有效的上下文指针
    unsafe fn cpu_context_sp(ctx_ptr: *const u8) -> usize;

    /// # Safety
    ///
    ///
    unsafe fn cpu_context_switch(prev: *mut u8, next: *mut u8);

    fn wait_for_interrupt();

    fn irq_all_enable();
    fn irq_all_disable();
    fn irq_all_is_enabled() -> bool;

    fn on_boot_success() {}
    fn shutdown() -> !;
    fn debug_put(b: u8);

    fn dcache_range(op: CacheOp, addr: usize, size: usize);

    fn driver_registers() -> DriverRegisterListRef;
}

#[cfg(feature = "mmu")]
#[api_trait]
pub trait MMU {
    fn set_kernel_table(addr: usize);
    fn get_kernel_table() -> usize;
    fn set_user_table(addr: usize);
    fn get_user_table() -> usize;

    /// flush tlb
    /// # Safety
    /// addr must be page aligned
    unsafe fn flush_tlb(addr: *const u8);
    fn flush_tlb_all();
    fn page_size() -> usize;
    fn table_level() -> usize;
    fn new_pte(config: PTEGeneric) -> usize;
    fn read_pte(pte: usize) -> PTEGeneric;
    fn enable_mmu(stack_top: usize, jump_to: usize) -> !;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum CacheOp {
    /// Write back to memory
    Clean,
    /// Invalidate cache
    Invalidate,
    /// Clean and invalidate
    CleanAndInvalidate,
}
