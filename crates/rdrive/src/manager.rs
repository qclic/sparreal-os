use core::ptr::NonNull;

use alloc::{boxed::Box, vec::Vec};
use driver_interface::DriverRegister;

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

    pub fn probe_by_fdt(&mut self, fdt: NonNull<u8>) -> Result<(), Box<dyn core::error::Error>> {
        

        Ok(())
    }
}
