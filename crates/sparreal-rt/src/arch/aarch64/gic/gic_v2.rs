use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
use core::{cell::UnsafeCell, error::Error, ptr::NonNull};

use arm_gic_driver::GicGeneric;
use sparreal_kernel::{
    driver_interface::{DriverGeneric, ProbeFnKind, RegAddress, interrupt_controller::*},
    mem::iomap,
};
use sparreal_macros::module_driver;

use super::*;

module_driver!(
    name: "GICv2",
    compatibles: &["arm,cortex-a15-gic", "arm,gic-400"],
    probe: ProbeFnKind::InterruptController(probe_gic_v2),
);

struct GicV2 {
    gicd: NonNull<u8>,
    gicc: NonNull<u8>,
    gic: Arc<UnsafeCell<Option<arm_gic_driver::GicV2>>>,
}

unsafe impl Send for GicV2 {}

impl GicV2 {
    #[allow(clippy::arc_with_non_send_sync)]
    fn new(gicd: NonNull<u8>, gicc: NonNull<u8>) -> Self {
        Self {
            gic: Arc::new(UnsafeCell::new(None)),
            gicd,
            gicc,
        }
    }
}

struct GicV2PerCpu(Arc<UnsafeCell<Option<arm_gic_driver::GicV2>>>);

unsafe impl Send for GicV2PerCpu {}

impl GicV2PerCpu {
    fn get_mut(&mut self) -> &mut arm_gic_driver::GicV2 {
        unsafe { &mut *self.0.get() }.as_mut().unwrap()
    }
}

impl InterfaceCPU for GicV2PerCpu {
    fn get_and_acknowledge_interrupt(&mut self) -> Option<IrqId> {
        self.get_mut()
            .get_and_acknowledge_interrupt()
            .map(|id| (id.to_u32() as usize).into())
    }

    fn end_interrupt(&mut self, irq: IrqId) {
        self.get_mut().end_interrupt(convert_id(irq));
    }

    fn irq_enable(&mut self, irq: IrqId) {
        self.get_mut().irq_enable(convert_id(irq));
    }

    fn irq_disable(&mut self, irq: IrqId) {
        self.get_mut().irq_disable(convert_id(irq));
    }

    fn set_priority(&mut self, irq: IrqId, priority: usize) {
        self.get_mut().set_priority(convert_id(irq), priority);
    }

    fn set_trigger(&mut self, irq: IrqId, trigger: Trigger) {
        self.get_mut().set_trigger(convert_id(irq), match trigger {
            Trigger::EdgeBoth => arm_gic_driver::Trigger::Edge,
            Trigger::EdgeRising => arm_gic_driver::Trigger::Edge,
            Trigger::EdgeFailling => arm_gic_driver::Trigger::Edge,
            Trigger::LevelHigh => arm_gic_driver::Trigger::Level,
            Trigger::LevelLow => arm_gic_driver::Trigger::Level,
        });
    }

    fn set_bind_cpu(&mut self, irq: IrqId, cpu_list: &[CpuId]) {
        let id_list = cpu_list
            .iter()
            .map(|v| arm_gic_driver::MPID::from(Into::<usize>::into(*v)))
            .map(|v| v.into())
            .collect::<Vec<_>>();

        self.get_mut().set_bind_cpu(convert_id(irq), &id_list);
    }

    fn parse_fdt_config(&self, prop_interupt: &[u32]) -> Result<IrqConfig, Box<dyn Error>> {
        fdt_parse_irq_config(prop_interupt)
    }

    fn irq_pin_to_id(&self, pin: usize) -> Result<IrqId, Box<dyn Error>> {
        super::irq_pin_to_id(pin)
    }
}

impl DriverGeneric for GicV2 {
    fn open(&mut self) -> Result<(), alloc::string::String> {
        let gic =
            arm_gic_driver::GicV2::new(self.gicd, self.gicc).map_err(|e| format!("{:?}", e))?;
        unsafe { &mut *self.gic.get() }.replace(gic);

        Ok(())
    }

    fn close(&mut self) -> Result<(), alloc::string::String> {
        Ok(())
    }
}

impl Interface for GicV2 {
    fn current_cpu_setup(&self) -> Box<dyn InterfaceCPU> {
        unsafe { &mut *self.gic.get() }
            .as_mut()
            .unwrap()
            .current_cpu_setup();
        Box::new(GicV2PerCpu(self.gic.clone()))
    }
}

fn probe_gic_v2(regs: Vec<RegAddress>) -> Hardware {
    let gicd_reg = regs[0];
    let gicc_reg = regs[1];
    let gicd = iomap(gicd_reg.addr.into(), gicd_reg.size.unwrap_or(0x1000));
    let gicc = iomap(gicc_reg.addr.into(), gicc_reg.size.unwrap_or(0x1000));

    Box::new(GicV2::new(gicd, gicc))
}
