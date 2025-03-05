use alloc::string::{String, ToString};
use alloc::vec::Vec;
use arrayvec::ArrayVec;
use core::{
    ops::Range,
    ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut},
};
use fdt_parser::{Node, Pci};
use rdrive::{Phandle, probe::ProbeData};

use crate::globals::{self, global_val};
use crate::irq::IrqInfo;
use crate::mem::{Align, VirtAddr};
use crate::{io::print::*, mem::PhysAddr};

use super::{CPUInfo, SerialPort};

#[derive(Clone)]
pub struct Fdt(NonNull<u8>);

impl Fdt {
    pub fn new(addr: NonNull<u8>) -> Self {
        Self(addr)
    }

    pub fn model_name(&self) -> Option<String> {
        let fdt = self.get();
        let node = fdt.all_nodes().next()?;

        let model = node.find_property("model")?;

        Some(model.str().to_string())
    }

    pub fn cpus(&self) -> Vec<CPUInfo> {
        let fdt = self.get();

        fdt.find_nodes("/cpus/cpu")
            .map(|cpu| {
                let reg = cpu.reg().unwrap().next().unwrap();
                CPUInfo {
                    cpu_id: (reg.address as usize).into(),
                }
            })
            .collect()
    }

    // pub fn setup(&mut self) -> Result<(), &'static str> {
    //     let main_memory = global_val().main_memory.clone();
    //     let fdt_start = self.move_to(main_memory.end.as_usize());
    //     unsafe { globals::edit(|g| g.kstack_top = fdt_start.into()) };
    //     Ok(())
    // }

    pub fn get(&self) -> fdt_parser::Fdt<'static> {
        fdt_parser::Fdt::from_ptr(self.0).unwrap()
    }

    pub fn get_addr(&self) -> NonNull<u8> {
        unsafe { NonNull::new_unchecked(VirtAddr::from(self.0).as_mut_ptr()) }
    }

    pub fn memorys(&self) -> ArrayVec<Range<crate::mem::addr2::PhysAddr>, 12> {
        let mut out = ArrayVec::new();

        let fdt = self.get();

        for node in fdt.memory() {
            for region in node.regions() {
                let addr = (region.address as usize).into();
                out.push(addr..addr + region.size);
            }
        } 
        out
    }

    pub fn take_memory(&self) -> Range<PhysAddr> {
        let region = self
            .get()
            .memory()
            .next()
            .unwrap()
            .regions()
            .next()
            .unwrap();
        let addr = (region.address as usize).into();
        addr..addr + region.size
    }

    pub fn debugcon(&self) -> Option<SerialPort> {
        let fdt = self.get();
        let stdout = fdt.chosen()?.stdout()?;
        let compatible = stdout.node.compatibles();
        let reg = stdout.node.reg()?.next()?;
        Some(SerialPort::new(
            (reg.address as usize).into(),
            reg.size,
            compatible,
        ))
    }
}

pub trait GetIrqConfig {
    fn irq_info(&self) -> Option<IrqInfo>;
}

impl GetIrqConfig for Node<'_> {
    fn irq_info(&self) -> Option<IrqInfo> {
        let irq_chip_node = self.interrupt_parent()?;
        let phandle = irq_chip_node.node.phandle()?;

        let interrupts = self.interrupts()?.map(|o| o.collect()).collect::<Vec<_>>();

        parse_irq_config(phandle, &interrupts)
    }
}

fn parse_irq_config(parent: Phandle, interrupts: &[Vec<u32>]) -> Option<IrqInfo> {
    let mut irq_parent = None;

    let mut cfgs = Vec::new();
    for raw in interrupts {
        match rdrive::read(|m| match &m.probe_kind {
            ProbeData::Fdt(probe_data) => {
                irq_parent = probe_data.phandle_2_device_id(parent);
                Some(probe_data.parse_irq(parent, raw))
            }
            ProbeData::Static => None,
        }) {
            Some(Ok(cfg)) => cfgs.push(cfg),
            _ => continue,
        }
    }

    Some(IrqInfo {
        irq_parent: irq_parent?,
        cfgs,
    })
}

pub trait GetPciIrqConfig {
    fn child_irq_info(&self, bus: u8, device: u8, function: u8, irq_pin: u8) -> Option<IrqInfo>;
}
impl GetPciIrqConfig for Pci<'_> {
    fn child_irq_info(&self, bus: u8, device: u8, func: u8, irq_pin: u8) -> Option<IrqInfo> {
        let irq = self
            .child_interrupts(bus, device, func, irq_pin as _)
            .ok()?;

        let raw = irq.irqs.collect::<Vec<_>>();

        parse_irq_config(irq.parent, &[raw])
    }
}
