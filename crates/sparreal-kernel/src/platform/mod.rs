use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::fmt::Display;
use core::ops::Range;
use core::ptr::NonNull;
use rdrive::register::DriverRegister;

use crate::globals::global_val;
use crate::mem::{Align, PhysAddr};
use crate::platform_if::*;
use fdt::Fdt;

pub mod fdt;

pub enum PlatformInfoKind {
    DeviceTree(Fdt),
}

impl PlatformInfoKind {
    pub fn new_fdt(addr: NonNull<u8>) -> Self {
        PlatformInfoKind::DeviceTree(Fdt::new(addr))
    }

    pub fn memorys(&self) -> impl Iterator<Item = Range<PhysAddr>> {
        let mut out: [Option<Range<PhysAddr>>; 24] =
            unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        let mut len = 0;

        match self {
            PlatformInfoKind::DeviceTree(fdt) => {
                for (i, m) in fdt
                    .get()
                    .memory()
                    .flat_map(|m| m.regions())
                    .map(|r| {
                        let start = PhysAddr::from(r.address as usize);
                        start..start + r.size
                    })
                    .enumerate()
                {
                    if i >= out.len() {
                        break;
                    }
                    out[i] = Some(m);
                    len += 1;
                }
            }
        }

        let mut iter = 0;
        core::iter::from_fn(move || {
            if iter >= len {
                None
            } else {
                let m = out[iter].take().unwrap();
                iter += 1;
                Some(m)
            }
        })
    }

    pub fn main_memory(&self) -> Option<Range<PhysAddr>> {
        let kernel_text = PlatformImpl::kernel_regions().text;

        let mut first = None;

        for m in self.memorys() {
            let r = m.start.as_usize()..m.end.as_usize();
            if r.contains(&kernel_text.end) {
                return Some(m);
            }
            if first.is_none() {
                first = Some(m);
            }
        }

        first
    }

    pub fn debugcon(&self) -> Option<SerialPort> {
        match self {
            Self::DeviceTree(fdt) => fdt.debugcon(),
        }
    }
}

pub fn cpu_list() -> Vec<CPUInfo> {
    match &global_val().platform_info {
        PlatformInfoKind::DeviceTree(fdt) => fdt.cpus(),
    }
}

pub fn cpu_hard_id() -> CPUHardId {
    PlatformImpl::cpu_id().into()
}

pub fn platform_name() -> String {
    match &global_val().platform_info {
        PlatformInfoKind::DeviceTree(fdt) => fdt.model_name().unwrap_or_default(),
    }
}

pub fn memory_main_available() -> Result<Range<crate::mem::addr2::PhysAddr>, &'static str> {
    let text = MMUImpl::rsv_regions()
        .into_iter()
        .find(|o| o.name().eq(".text"))
        .ok_or("can not find .text")?;
    let text_end = text.range.end;

    let main_memory = phys_memorys()
        .into_iter()
        .find(|m| m.contains(&text_end))
        .ok_or("can not find main memory")?;

    let mut start = crate::mem::addr2::PhysAddr::new(0);
    for rsv in MMUImpl::rsv_regions() {
        if main_memory.contains(&rsv.range.end) && rsv.range.end > start {
            start = rsv.range.end;
        }
    }
    start = start.align_up(0x1000);
    Ok(start..main_memory.end)
}

pub fn phys_memorys() -> ArrayVec<Range<crate::mem::addr2::PhysAddr>, 12> {
    match &global_val().platform_info {
        PlatformInfoKind::DeviceTree(fdt) => fdt.memorys(),
    }
}

pub fn shutdown() -> ! {
    PlatformImpl::shutdown()
}

pub fn wait_for_interrupt() {
    PlatformImpl::wait_for_interrupt();
}

pub fn kstack_size() -> usize {
    PlatformImpl::kstack_size()
}

pub fn page_size() -> usize {
    #[cfg(feature = "mmu")]
    {
        MMUImpl::page_size()
    }

    #[cfg(not(feature = "mmu"))]
    {
        0x1000
    }
}

pub fn app_main() {
    unsafe extern "C" {
        fn __sparreal_rt_main();
    }
    unsafe { __sparreal_rt_main() }
}

#[derive(Debug)]
pub struct CPUInfo {
    pub cpu_id: CPUHardId,
}

#[derive(Debug, Clone, Copy)]
pub struct SerialPort {
    pub addr: PhysAddr,
    pub size: Option<usize>,
    compatible: [Option<[u8; 128]>; 4],
}

impl SerialPort {
    pub fn new<'a>(
        addr: PhysAddr,
        size: Option<usize>,
        compatibles: impl Iterator<Item = &'a str>,
    ) -> Self {
        let mut compatible_out = [None; 4];

        for (i, c) in compatibles.enumerate() {
            if i == compatible_out.len() {
                break;
            }
            let bytes = c.as_bytes();
            let mut bytes_out = [0u8; 128];
            bytes_out[..bytes.len()].copy_from_slice(bytes);
            compatible_out[i] = Some(bytes_out);
        }

        Self {
            addr,
            size,
            compatible: compatible_out,
        }
    }

    pub fn compatibles(&self) -> impl Iterator<Item = &str> {
        let mut iter = 0;

        core::iter::from_fn(move || {
            if iter >= self.compatible.len() {
                None
            } else {
                let bytes = self.compatible[iter].as_ref()?;
                iter += 1;
                CStr::from_bytes_until_nul(bytes).ok()?.to_str().ok()
            }
        })
    }
}

pub fn module_registers() -> Vec<DriverRegister> {
    PlatformImpl::driver_registers().as_slice().to_vec()
}

pub type CPUHardId = rdrive::intc::CpuId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct CPUId(usize);
impl Display for CPUId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
