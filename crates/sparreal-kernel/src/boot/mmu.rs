use core::sync::atomic::{Ordering, fence};

use super::{__start, print_info};
use crate::{
    globals::{self, global_val},
    io::print::*,
    mem::{self, mmu::*, set_va_offset_now, stack_top},
    platform::PlatformInfoKind,
    platform_if::MMUImpl,
};

pub fn start(text_va_offset: usize, platform_info: PlatformInfoKind) -> Result<(), &'static str> {
    early_dbgln("Booting up");
    mem::set_text_va_offset(text_va_offset);
    if let Err(e) = unsafe { globals::setup(platform_info) } {
        early_dbgln("setup globle error: ");
        early_dbgln(e);
    }
    let table = new_boot_table()?;

    fence(Ordering::SeqCst);

    set_user_table(table);
    set_kernel_table(table);

    flush_tlb_all();

    fence(Ordering::SeqCst);

    let jump_to = __start as usize + text_va_offset;
    // let stack_top = global_val().kstack_top.as_usize() + text_va_offset;
    unsafe { set_va_offset_now(text_va_offset) };

    early_dbg("Jump to __start: ");
    early_dbg_hex(jump_to as _);
    early_dbg(", stack top: ");
    early_dbg_hexln(stack_top() as _);

    MMUImpl::enable_mmu(stack_top(), jump_to)
}
