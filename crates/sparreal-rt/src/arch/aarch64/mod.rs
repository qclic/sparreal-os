mod boot;
mod cache;
mod context;
mod debug;
mod mmu;
mod psci;
mod trap;

use core::{arch::asm, ffi::c_void};

use aarch64_cpu::registers::*;
use context::CpuContext;
use sparreal_kernel::{driver::device_tree::get_device_tree, platform::Platform, print, println};
use sparreal_macros::api_impl;

const CPU_ID_MASK: u64 = 0xFF_FFFF + (0xFFFF_FFFF << 32);

pub struct PlatformImpl;

#[api_impl]
impl Platform for PlatformImpl {
    unsafe fn shutdown() {
        psci::system_off();
    }

    unsafe fn wait_for_interrupt() {
        aarch64_cpu::asm::wfi();
    }

    unsafe fn current_ticks() -> u64 {
        CNTPCT_EL0.get()
    }

    unsafe fn tick_hz() -> u64 {
        CNTFRQ_EL0.get()
    }

    unsafe fn debug_write_byte(_value: u8) {
        #[cfg(feature = "early-print")]
        debug::put_debug(_value);
    }

    fn print_system_info() {
        println!(
            "CPU: {}.{}.{}.{}",
            MPIDR_EL1.read(MPIDR_EL1::Aff0),
            MPIDR_EL1.read(MPIDR_EL1::Aff1),
            MPIDR_EL1.read(MPIDR_EL1::Aff2),
            MPIDR_EL1.read(MPIDR_EL1::Aff3)
        );
        let _ = print_board_info();
    }

    fn irqs_enable() {
        unsafe { asm!("msr daifclr, #2") };
    }

    fn irqs_disable() {
        unsafe { asm!("msr daifset, #2") };
    }

    fn cpu_id() -> u64 {
        MPIDR_EL1.get() & CPU_ID_MASK
    }

    fn get_current_tcb_addr() -> *mut u8 {
        SP_EL0.get() as usize as _
    }
    fn set_current_tcb_addr(addr: *mut u8) {
        SP_EL0.set(addr as usize as _);
    }
    fn task_cpu_context_size() -> usize {
        size_of::<CpuContext>()
    }
    fn cpu_context_init(ctx_ptr: *mut u8, pc: *mut c_void, stack_top: *mut u8) {
        unsafe {
            let ctx = &mut *(ctx_ptr as *mut CpuContext);
            ctx.pc = pc as usize as _;
            ctx.sp = stack_top as usize as _;
        }
    }
    fn cpu_context_switch(prev: *mut u8, next: *mut u8) {
        unsafe {
            let prev = &mut *(prev as *mut CpuContext);
            let next = &mut *(next as *mut CpuContext);
            prev.switch_to(next);
        }
    }
}

fn print_board_info() -> Option<()> {
    let fdt = get_device_tree()?;
    let root = fdt.all_nodes().next()?;
    let caps = root.compatibles();

    print!("Board:");
    for cap in caps {
        print!(" {}", cap);
    }
    println!();
    Some(())
}
