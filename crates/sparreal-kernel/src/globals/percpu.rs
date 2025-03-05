use core::fmt::Display;

use alloc::collections::btree_map::BTreeMap;
use rdrive::intc::CpuId;

use crate::{
    irq,
    platform::{CPUHardId, CPUId},
    time::TimerData,
};

use super::once::OnceStatic;

static HARD_TO_SOFT: OnceStatic<BTreeMap<CPUHardId, CPUId>> = OnceStatic::new(BTreeMap::new());
static SOFT_TO_HARD: OnceStatic<BTreeMap<CPUId, CPUHardId>> = OnceStatic::new(BTreeMap::new());
static PER_CPU: OnceStatic<BTreeMap<CPUId, PerCPU>> = OnceStatic::new(BTreeMap::new());

impl From<CPUHardId> for CPUId {
    fn from(value: CPUHardId) -> Self {
        unsafe { *(*HARD_TO_SOFT.get()).get(&value).unwrap() }
    }
}

impl From<CPUId> for CPUHardId {
    fn from(value: CPUId) -> Self {
        unsafe { *(*SOFT_TO_HARD.get()).get(&value).unwrap() }
    }
}

#[derive(Default)]
pub struct PerCPU {
    pub irq_chips: irq::CpuIrqChips,
    pub timer: TimerData,
}
