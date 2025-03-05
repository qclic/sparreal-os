#![allow(unused)]

use core::{
    cell::UnsafeCell,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::collections::btree_map::BTreeMap;
use log::debug;

pub use crate::platform::PlatformInfoKind;
use crate::{
    mem::{self, PhysAddr},
    platform::{self, CPUHardId, CPUId, cpu_list, fdt::Fdt},
};

mod once;
mod percpu;

pub(crate) use percpu::*;

pub struct GlobalVal {
    pub platform_info: PlatformInfoKind,
    pub main_memory: Range<PhysAddr>,
    percpu: BTreeMap<CPUId, percpu::PerCPU>,
}

struct LazyGlobal {
    g_ok: AtomicBool,
    g: UnsafeCell<Option<GlobalVal>>,
}

unsafe impl Sync for LazyGlobal {}

static GLOBAL: LazyGlobal = LazyGlobal::new();

pub fn global_val() -> &'static GlobalVal {
    global_val_meybeuninit().expect("GlobalVal is not init!")
}

pub fn global_val_meybeuninit() -> Option<&'static GlobalVal> {
    if !GLOBAL.g_ok.load(Ordering::SeqCst) {
        return None;
    }
    Some(unsafe { (*GLOBAL.g.get()).as_ref().unwrap() })
}

impl LazyGlobal {
    const fn new() -> Self {
        Self {
            g_ok: AtomicBool::new(false),
            g: UnsafeCell::new(None),
        }
    }
}

/// 修改全局变量
///
/// # Safty
/// 只能在其他CPU启动前调用
pub(crate) unsafe fn edit(f: impl FnOnce(&mut GlobalVal)) {
    unsafe {
        let global = (*GLOBAL.g.get()).as_mut().unwrap();
        f(global);
    }
}

unsafe fn get_mut() -> &'static mut GlobalVal {
    unsafe { (*GLOBAL.g.get()).as_mut().unwrap() }
}

#[cfg(feature = "mmu")]
pub(crate) unsafe fn mmu_relocate() {
    unsafe {
        edit(|g| match &g.platform_info {
            PlatformInfoKind::DeviceTree(fdt) => {
                let addr = fdt.get_addr();
                let vaddr = addr.add(mem::mmu::LINER_OFFSET);
                g.platform_info = PlatformInfoKind::DeviceTree(Fdt::new(vaddr))
            }
        });
    }
}

/// # Safty
/// 只能在其他CPU启动前调用
pub(crate) unsafe fn setup(platform_info: PlatformInfoKind) -> Result<(), &'static str> {
    let main_memory = platform::memory_main_available(&platform_info)?;

    let g = GlobalVal {
        platform_info,
        main_memory,
        percpu: Default::default(),
    };

    unsafe {
        GLOBAL.g.get().write(Some(g));
        GLOBAL.g_ok.store(true, Ordering::SeqCst);
    }
    Ok(())
}




