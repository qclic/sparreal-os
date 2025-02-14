#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod device;
mod id;
mod manager;

pub use device::*;
pub use manager::*;
pub mod error;
pub mod register;

use spin::Mutex;

static MANAGER: Mutex<Manager> = Mutex::new(Manager::new());
