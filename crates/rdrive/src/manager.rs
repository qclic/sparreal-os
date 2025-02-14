use alloc::collections::BTreeMap;
use core::ptr::NonNull;

use crate::{
    Device, DeviceId,
    register::{DriverRegister, OnProbeKindFdt},
};
use alloc::{string::*, vec::Vec};
use fdt_parser::{Fdt, Node};

use crate::{error::DriverError, register::ProbeKind};

use super::device;

#[derive(Default)]
pub struct Manager {
    registers: Vec<DriverRegister>,
    pub intc: device::intc::Container,
}

impl Manager {
    pub const fn new() -> Self {
        Self {
            registers: Vec::new(),
            intc: device::Container::new(),
        }
    }

    pub fn append_register(&mut self, register: &[DriverRegister]) {
        self.registers.extend_from_slice(register);
    }

    pub fn add_register(&mut self, register: DriverRegister) {
        self.registers.push(register);
    }

    fn probe_by_fdt(&mut self, fdt: NonNull<u8>) -> Result<(), DriverError> {
        let fdt = fdt_parser::Fdt::from_ptr(fdt)?;
        let registers = self.get_all_fdt_registers(&fdt)?;
        let mut irq_parse = BTreeMap::new();

        for register in registers {
            match register.on_probe {
                OnProbeKindFdt::InterruptController(on_probe) => {
                    let info = on_probe(register.node.clone())?;
                    irq_parse.insert(
                        register
                            .phandle
                            .ok_or(DriverError::Fdt("intc no phandle".to_string()))?,
                        info.fdt_parse_config_fn,
                    );
                    self.intc
                        .insert(Device::new(register.descriptor, info.hardware));
                }
                OnProbeKindFdt::Timer(on_probe) => todo!(),
            }
        }

        Ok(())
    }

    fn get_all_fdt_registers<'a>(
        &self,
        fdt: &'a Fdt<'_>,
    ) -> Result<Vec<ProbeFdtInfo<'a>>, DriverError> {
        let mut vec = Vec::new();
        for node in fdt.all_nodes() {
            for register in self.registers.clone() {
                for probe in register.probe_kinds {
                    match probe {
                        ProbeKind::Fdt {
                            compatibles,
                            on_probe,
                        } => {
                            if let Some(node_campatibles) = node.compatible() {
                                for campatible in node_campatibles {
                                    if let Ok(campatible) = campatible {
                                        if compatibles.contains(&campatible) {
                                            vec.push(ProbeFdtInfo {
                                                node: node.clone(),
                                                on_probe: on_probe.clone(),
                                                descriptor: device::Descriptor {
                                                    name: register.name,
                                                    device_id: DeviceId::new(),
                                                    ..Default::default()
                                                },
                                                phandle: node.phandle().map(|p| p.as_usize()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(vec)
    }
}

struct ProbeIterFdt<'a> {
    m: &'a mut Manager,
    fdt: fdt_parser::Fdt<'a>,
}

struct ProbeFdtInfo<'a> {
    node: Node<'a>,
    on_probe: OnProbeKindFdt,
    descriptor: device::Descriptor,
    phandle: Option<usize>,
}
