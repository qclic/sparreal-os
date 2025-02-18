#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub(crate) mod _macro;
pub(crate) mod err;
pub mod intc;
pub mod io;
pub mod timer;
pub mod uart;
pub use err::{DriverError, DriverResult};
pub use intc::IrqConfig;
pub mod lock;

pub trait DriverGeneric: Send {
    fn open(&mut self) -> DriverResult;
    fn close(&mut self) -> DriverResult;
}

#[derive(Debug, Clone, Copy)]
pub struct RegAddress {
    pub addr: usize,
    pub size: Option<usize>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IrqHandleResult {
    Handled,
    Unhandled,
}
