use core::{error::Error, fmt::Debug};

use crate::{DriverGeneric, custom_type};
use alloc::boxed::Box;

custom_type!(IrqId, usize, "{:#x}");
custom_type!(CpuId, usize, "{:#x}");

pub type Hardware = Box<dyn Interface>;
pub type HardwareCPU = Box<dyn InterfaceCPU>;

/// Fdt 解析 `interrupts` 函数，一次解析一个`cell`
pub type FdtParseConfigFn =
    fn(prop_interrupts_one_cell: &[u32]) -> Result<IrqConfig, Box<dyn Error>>;

pub struct FdtProbeInfo {
    pub hardware: Hardware,
    pub fdt_parse_config_fn: FdtParseConfigFn,
}

pub trait InterfaceCPU: Send + Sync {
    fn get_and_acknowledge_interrupt(&self) -> Option<IrqId>;
    fn end_interrupt(&self, irq: IrqId);
}

pub trait Interface: DriverGeneric {
    fn current_cpu_setup(&self) -> HardwareCPU;
    fn irq_enable(&mut self, irq: IrqId);
    fn irq_disable(&mut self, irq: IrqId);
    fn set_priority(&mut self, irq: IrqId, priority: usize);
    fn set_trigger(&mut self, irq: IrqId, trigger: Trigger);
    fn set_target_cpu(&mut self, irq: IrqId, cpu: CpuId);
}

/// The trigger configuration for an interrupt.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Trigger {
    EdgeBoth,
    EdgeRising,
    EdgeFailling,
    LevelHigh,
    LevelLow,
}

#[derive(Debug, Clone)]
pub struct IrqConfig {
    pub irq: IrqId,
    pub trigger: Trigger,
}
