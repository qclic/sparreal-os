use core::ptr::NonNull;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use driver_interface::{DriverRegister, OnProbeKindFdt, timer::*};
use fdt_parser::Fdt;
use log::debug;

use crate::prelude::GetIrqConfig;

use super::{Descriptor, Device};

pub struct Container {
    data: Option<Device<Hardware>>,
}

impl Container {
    pub const fn new() -> Self {
        Self { data: None }
    }

    pub fn set(&mut self, device: Device<Hardware>) {
        self.data = Some(device);
    }

    pub fn get_cpu_timer(&self) -> Option<Device<Timer>> {
        if let Some(device) = self.data.as_ref() {
            loop {
                if let Ok(mut d) = device.try_use_by("cpu") {
                    let p = d.get_current_cpu();
                    let mut desc = d.descriptor.clone();
                    desc.device_id = Default::default();
                    return Some(Device::new(desc, Timer::new(p)));
                }
            }
        }

        None
    }
}

pub fn init_by_fdt(
    registers: &[DriverRegister],
    fdt_addr: NonNull<u8>,
) -> Result<Device<Hardware>, String> {
    let fdt = Fdt::from_ptr(fdt_addr).map_err(|e| format!("{e:?}"))?;
    for r in registers {
        for kind in r.probe_kinds {
            match kind {
                driver_interface::ProbeKind::Fdt {
                    compatibles,
                    on_probe,
                } => {
                    if let OnProbeKindFdt::Timer(probe) = on_probe {
                        let compa = compatibles
                            .iter()
                            .filter_map(|e| if e.is_empty() { None } else { Some(*e) })
                            .collect::<Vec<_>>();
                        for node in fdt.find_compatible(&compa) {
                            let irq = match node.irq_info() {
                                Some(irq) => irq,
                                None => continue,
                            };

                            let timer = probe(node);
                            debug!("[{}] ok", r.name);
                            let dev = Device::new(
                                Descriptor {
                                    name: r.name.to_string(),
                                    irq: Some(irq),
                                    ..Default::default()
                                },
                                timer,
                            );

                            return Ok(dev);
                        }
                    }
                }
            }
        }
    }
    Err("No timer found".into())
}
