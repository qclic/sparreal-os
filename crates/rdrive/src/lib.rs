#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::ptr::NonNull;
use error::DriverError;
pub use fdt_parser::Phandle;

use spin::Mutex;

mod device;
pub mod error;
mod id;
mod manager;
mod probe;
pub mod register;
pub use device::*;
pub use manager::*;
pub use register::{DriverRegister, DriverRegisterSlice};

static MANAGER: Mutex<Option<Manager>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub enum DriverInfoKind {
    Fdt { addr: NonNull<u8> },
    Static,
}

unsafe impl Send for DriverInfoKind {}

pub fn init(probe_kind: DriverInfoKind) {
    MANAGER.lock().replace(Manager::new(probe_kind));
}

fn write<F, T>(f: F) -> T
where
    F: FnOnce(&mut Manager) -> T,
{
    let mut g = MANAGER.lock();
    f(g.as_mut().expect("manager not init"))
}

fn read<F, T>(f: F) -> T
where
    F: FnOnce(&Manager) -> T,
{
    let g = MANAGER.lock();
    f(g.as_ref().expect("manager not init"))
}

pub fn register_add(register: DriverRegister) {
    write(|manager| manager.registers.add(register));
}

pub fn register_append(registers: &[DriverRegister]) {
    write(|manager| manager.registers.append(registers))
}

pub fn probe() -> Result<(), DriverError> {
    write(|manager| manager.probe())
}

pub fn probe_intc() -> Result<(), DriverError> {
    write(|manager| manager.probe_intc())
}

pub fn probe_timer() -> Result<(), DriverError> {
    write(|manager| manager.probe_timer())
}

pub fn intc_all() -> Vec<(DeviceId, device::intc::Weak)> {
    read(|manager| manager.intc.all())
}

pub fn intc_get(id: DeviceId) -> Option<device::intc::Weak> {
    read(|manager| manager.intc.get(id))
}
