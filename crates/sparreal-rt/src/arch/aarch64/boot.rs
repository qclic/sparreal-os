use core::{
    arch::{asm, global_asm, naked_asm},
    ptr::NonNull,
};

use crate::mem::clear_bss;
use aarch64_cpu::registers::*;
use sparreal_kernel::{io::print::*, platform::PlatformInfoKind};

global_asm!(include_str!("boot.s"));

const FLAG_LE: usize = 0b0;
const FLAG_PAGE_SIZE_4K: usize = 0b10;
const FLAG_ANY_MEM: usize = 0b1000;

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.head")]
/// The entry point of the kernel.
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            // code0/code1
            "nop",
            "bl {entry}",
            // text_offset
            ".quad 0",
            // image_size
            ".quad _kernel_size",
            // flags
            ".quad {flags}",
            // Reserved fields
            ".quad 0",
            ".quad 0",
            ".quad 0",
            // magic - yes 0x644d5241 is the same as ASCII string "ARM\x64"
            ".ascii \"ARM\\x64\"",
            // Another reserved field at the end of the header
            ".byte 0, 0, 0, 0",
            flags = const FLAG_LE | FLAG_PAGE_SIZE_4K | FLAG_ANY_MEM,
            entry = sym primary_entry,
        )
    }
}

#[naked]
#[unsafe(link_section = ".text.boot")]
/// The entry point of the kernel.
unsafe extern "C" fn primary_entry() -> ! {
    unsafe {
        naked_asm!(
            "ADR      x11, .",
            "LDR      x10, ={this_func}",
            "SUB      x18, x10, x11", // x18 = va_offset
            "MOV      x19, x0",        // x19 = dtb_addr

            "LDR      x1, =_stack_top",
            "SUB      x1, x1, x18", // X1 == STACK_TOP
            "MOV      sp, x1",
            "BL       {switch_to_el1}",
            this_func = sym primary_entry,
            switch_to_el1 = sym switch_to_el1,
        )
    }
}

#[unsafe(no_mangle)]
extern "C" fn __rust_boot(va_offset: usize, fdt_addr: usize) {
    unsafe {
        clear_bss();

        let platform_info: PlatformInfoKind = if let Some(addr) = NonNull::new(fdt_addr as _) {
            PlatformInfoKind::new_fdt(addr)
        } else {
            todo!()
        };

        if let Some(info) = platform_info.debugcon() {
            crate::debug::init_by_info(info);
        }

        let rsv = [];

        if let Err(s) = sparreal_kernel::boot::start(va_offset, platform_info, &rsv) {
            early_dbgln(s);
        }
    }
}

fn switch_to_el1() {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el >= 2 {
        if current_el == 3 {
            // Set EL2 to 64bit and enable the HVC instruction.
            SCR_EL3.write(
                SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
            );
            // Set the return address and exception level.
            SPSR_EL3.write(
                SPSR_EL3::M::EL1h
                    + SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked,
            );
            unsafe {
                asm!(
                "adr      x2, {}",
                "msr elr_el3, x2",
                 sym primary_entry
                    );
            }
        }
        // Disable EL1 timer traps and the timer offset.
        CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        CNTVOFF_EL2.set(0);
        // Set EL1 to 64bit.
        HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
        // Set the return address and exception level.
        SPSR_EL2.write(
            SPSR_EL2::M::EL1h
                + SPSR_EL2::D::Masked
                + SPSR_EL2::A::Masked
                + SPSR_EL2::I::Masked
                + SPSR_EL2::F::Masked,
        );
        unsafe {
            asm!(
                "
            mov     x8, sp
            msr     sp_el1, x8
            MOV      x0, x19
            adr      x2, _el1_entry
            msr      elr_el2, x2
            eret
            "
            )
        };
    } else {
        unsafe { asm!("bl _el1_entry") }
    }
}
