#![allow(unused)]

use ansi_rgb::{Foreground, orange};
use log::LevelFilter;

use crate::{
    driver,
    globals::{self, global_val},
    io::{self, print::*},
    irq,
    logger::KLogger,
    mem::{self, VirtAddr, region, stack_top},
    platform::{self, app_main, module_registers, platform_name, shutdown},
    platform_if::*,
    println, task, time,
};

pub mod debug;

#[cfg(feature = "mmu")]
mod mmu;

#[cfg(feature = "mmu")]
pub use mmu::start;

#[repr(align(0x10))]
pub extern "C" fn __start() -> ! {
    early_dbgln("Relocate success.");

    io::print::stdout_use_debug();

    let _ = log::set_logger(&KLogger);
    log::set_max_level(LevelFilter::Trace);

    mem::init_heap();

    unsafe { globals::setup_percpu() };

    print_start_msg();

    mem::init_page_and_memory();

    driver::init();

    irq::enable_all();

    task::init();

    driver::probe();

    app_main();

    shutdown()
}

macro_rules! print_pair {
    ($name:expr, $($arg:tt)*) => {
        $crate::print!("{:<30}: {}\r\n", $name, format_args!($($arg)*));
    };
}

fn print_start_msg() {
    println!("{}", LOGO.fg(orange()));

    print_pair!("Version", env!("CARGO_PKG_VERSION"));
    print_pair!("Platfrom", "{}", platform_name());
    // print_pair!("Kernel Base", "{:p}", region::text().as_ptr());

    // let size =
    //     region::bss().as_ptr_range().end as usize - region::text().as_ptr_range().start as usize;

    // print_pair!("Kernel Size", "{:#}", byte_unit::Byte::from_u64(size as _));
    print_pair!("Kernel Stack Top", "{}", VirtAddr::from(stack_top()));

    print_pair!("Start CPU", "{}", platform::cpu_hard_id());

    if let Some(debug) = global_val().platform_info.debugcon() {
        if let Some(c) = debug.compatibles().next() {
            print_pair!("Debug Serial", "{}", c);
        }
    }
}

static LOGO: &str = r#"
     _____                                         __
    / ___/ ____   ____ _ _____ _____ ___   ____ _ / /
    \__ \ / __ \ / __ `// ___// ___// _ \ / __ `// / 
   ___/ // /_/ // /_/ // /   / /   /  __// /_/ // /  
  /____// .___/ \__,_//_/   /_/    \___/ \__,_//_/   
       /_/                                           
"#;
