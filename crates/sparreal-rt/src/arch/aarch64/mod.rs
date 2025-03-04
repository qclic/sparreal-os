use core::arch::asm;

use aarch64_cpu::registers::*;
use sparreal_kernel::{mem::KernelRegions, platform_if::*};

mod boot;
mod debug;
mod paging;

pub fn is_mmu_enabled() -> bool {
    SCTLR_EL2.matches_any(&[SCTLR_EL2::M::Enable])
}

struct PlatformImpl;

#[api_impl]
impl Platform for PlatformImpl {
    fn kstack_size() -> usize {
        // crate::config::KERNEL_STACK_SIZE
        todo!()
    }

    fn kernel_regions() -> KernelRegions {
        // crate::mem::kernel_regions()
        todo!()
    }

    fn cpu_id() -> usize {
        // cpu_id()
        todo!()
    }

    fn cpu_context_size() -> usize {
        // size_of::<Context>()
        todo!()
    }

    unsafe fn cpu_context_sp(ctx_ptr: *const u8) -> usize {
        // let ctx: &Context = unsafe { &*(ctx_ptr as *const Context) };
        // ctx.sp as _

        0
    }

    unsafe fn get_current_tcb_addr() -> *mut u8 {
        SP_EL0.get() as usize as _
    }

    unsafe fn set_current_tcb_addr(addr: *mut u8) {
        SP_EL0.set(addr as usize as _);
    }

    /// # Safety
    ///
    /// `ctx_ptr` 是有效的上下文指针
    unsafe fn cpu_context_set_sp(ctx_ptr: *const u8, sp: usize) {
        // unsafe {
        //     let ctx = &mut *(ctx_ptr as *mut Context);
        //     ctx.sp = sp as _;
        // }
    }

    /// # Safety
    ///
    /// `ctx_ptr` 是有效的上下文指针
    unsafe fn cpu_context_set_pc(ctx_ptr: *const u8, pc: usize) {
        // unsafe {
        //     let ctx = &mut *(ctx_ptr as *mut Context);
        //     ctx.pc = pc as _;
        //     ctx.lr = pc as _;
        // }
    }

    unsafe fn cpu_context_switch(prev_ptr: *mut u8, next_ptr: *mut u8) {
        // let next = TaskControlBlock::from(next_ptr);
        // trace!("switch to: {:?}", unsafe { &*(next.sp as *const Context) });
        // unsafe { __tcb_switch(prev_ptr, next_ptr) };
    }

    fn wait_for_interrupt() {
        aarch64_cpu::asm::wfi();
    }

    fn shutdown() -> ! {
        // psci::system_off()
        todo!()
    }

    fn debug_put(b: u8) {
        debug::put(b);
    }

    fn irq_all_enable() {
        unsafe { asm!("msr daifclr, #2") };
    }
    fn irq_all_disable() {
        unsafe { asm!("msr daifset, #2") };
    }
    fn irq_all_is_enabled() -> bool {
        !DAIF.is_set(DAIF::I)
    }

    fn on_boot_success() {
        // match &global_val().platform_info {
        //     PlatformInfoKind::DeviceTree(fdt) => {
        //         if let Err(e) = psci::setup_method_by_fdt(fdt.get()) {
        //             println!("{}", e);
        //         }
        //     }
        // }
    }

    fn dcache_range(op: CacheOp, addr: usize, size: usize) {
        // cache::dcache_range(op, addr, size);
    }

    fn driver_registers() -> DriverRegisterSlice {
        // DriverRegisterSlice::from_raw(driver_registers())
        todo!()
    }
}
