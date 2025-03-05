use alloc::{string::String, vec::Vec};
use core::{ffi::CStr, fmt::Display, ops::Range, ptr::NonNull};

use fdt::Fdt;
use rdrive::register::DriverRegister;

use crate::globals::global_val;
use crate::mem::PhysAddr;
use crate::platform_if::*;

pub mod fdt;

#[derive(Clone)]
pub enum PlatformInfoKind {
    DeviceTree(Fdt),
}

unsafe impl Send for PlatformInfoKind {}

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
    CPUHardId(PlatformImpl::cpu_id())
}

pub fn platform_name() -> String {
    match &global_val().platform_info {
        PlatformInfoKind::DeviceTree(fdt) => fdt.model_name().unwrap_or_default(),
    }
}

pub fn memory_main_available(
    platform_info: &PlatformInfoKind,
) -> Result<Range<PhysAddr>, &'static str> {
    let text = MMUImpl::rsv_regions()
        .into_iter()
        .find(|o| o.name().eq(".text"))
        .ok_or("can not find .text")?;
    let text_end = text.range.end;

    let main_memory = platform_info
        .memorys()
        .find(|m| m.contains(&text_end))
        .ok_or("can not find main memory")?;

    let mut start = PhysAddr::new(0);
    for rsv in MMUImpl::rsv_regions() {
        if main_memory.contains(&rsv.range.end) && rsv.range.end > start {
            start = rsv.range.end;
        }
    }
    start = start.align_up(0x1000);
    Ok(start..main_memory.end)
}

pub fn regsions() -> Vec<RsvRegion> {
    let mut ret = MMUImpl::rsv_regions().to_vec();
    let main_available = memory_main_available(&global_val().platform_info).unwrap();
    ret.push(RsvRegion::new(
        main_available.clone(),
        c"main mem",
        AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
        CacheSetting::Normal,
        RegionKind::Other,
    ));

    for memory in phys_memorys() {
        if memory.contains(&main_available.start) {
            continue;
        }

        ret.push(RsvRegion::new(
            memory,
            c"memory",
            AccessSetting::Read | AccessSetting::Write | AccessSetting::Execute,
            CacheSetting::Normal,
            RegionKind::Other,
        ));
    }

    ret
}

pub fn phys_memorys() -> ArrayVec<Range<PhysAddr>, 12> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct CPUId(usize);
impl Display for CPUId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl From<CPUId> for usize {
    fn from(value: CPUId) -> Self {
        value.0
    }
}
impl From<usize> for CPUId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct CPUHardId(usize);

impl CPUHardId {
    pub(crate) unsafe fn new(id: usize) -> Self {
        Self(id)
    }
}

impl Display for CPUHardId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl From<rdrive::intc::CpuId> for CPUHardId {
    fn from(value: rdrive::intc::CpuId) -> Self {
        Self(value.into())
    }
}

impl From<CPUHardId> for rdrive::intc::CpuId {
    fn from(value: CPUHardId) -> Self {
        Self::from(value.0)
    }
}
