use core::error::Error;

use alloc::{boxed::Box, format};
use arm_gic_driver::v3::Gic;
use fdt_parser::Node;
use sparreal_kernel::{
    driver_interface::{OnProbeKindFdt, ProbeKind, intc},
    mem::iomap,
};
use sparreal_macros::module_driver;

module_driver!(
    name: "GICv3",
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,gic-v3"],
            on_probe: OnProbeKindFdt::InterruptController(probe_gic)
        }
    ]
);

fn probe_gic(node: Node<'_>) -> Result<intc::FdtProbeInfo, Box<dyn Error>> {
    let mut reg = node.reg().ok_or(format!("[{}] has no reg", node.name))?;

    let gicd_reg = reg.next().unwrap();
    let gicc_reg = reg.next().unwrap();
    let gicd = iomap(
        (gicd_reg.address as usize).into(),
        gicd_reg.size.unwrap_or(0x1000),
    );
    let gicr = iomap(
        (gicc_reg.address as usize).into(),
        gicc_reg.size.unwrap_or(0x1000),
    );

    Ok(intc::FdtProbeInfo {
        hardware: Box::new(Gic::new(gicd, gicr, Default::default())),
        fdt_parse_config_fn: fdt_parse_irq_config,
    })
}
