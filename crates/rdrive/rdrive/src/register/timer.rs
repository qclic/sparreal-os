use alloc::boxed::Box;
use core::error::Error;

pub use driver_interface::timer::Hardware;

pub type OnProbeFdt = fn(node: super::FdtInfo<'_>) -> Result<Hardware, Box<dyn Error>>;
