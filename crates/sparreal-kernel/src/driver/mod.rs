use log::debug;
pub use rdrive::*;

use crate::{globals::global_val, irq, platform, time};

pub fn init() {
    let info = match &global_val().platform_info {
        crate::globals::PlatformInfoKind::DeviceTree(fdt) => DriverInfoKind::Fdt {
            addr: fdt.get_addr(),
        },
    };

    rdrive::init(info);

    rdrive::register_append(&platform::module_registers());

    debug!("add registers");

    rdrive::probe_intc().unwrap();

    irq::init_main_cpu();

    rdrive::probe_timer().unwrap();

    time::init_current_cpu();
}

pub fn probe() {
    rdrive::probe().unwrap();
}
