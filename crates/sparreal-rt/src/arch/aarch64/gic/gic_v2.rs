use alloc::{boxed::Box, format};
use core::error::Error;
use fdt_parser::Node;

use arm_gic_driver::{fdt_parse_irq_config, v2::Gic};
use sparreal_kernel::{
    driver::{
        module_driver,
        register::{intc::FdtProbeInfo, *},
    },
    mem::iomap,
};

module_driver!(
    name: "GICv2",
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,cortex-a15-gic", "arm,gic-400"],
            on_probe: OnProbeKindFdt::Intc(probe_gic)
        },
    ] ,
);

fn probe_gic(node: Node<'_>) -> Result<FdtProbeInfo, Box<dyn Error>> {
    let mut reg = node.reg().ok_or(format!("[{}] has no reg", node.name))?;

    let gicd_reg = reg.next().unwrap();
    let gicc_reg = reg.next().unwrap();
    let gicd = iomap(
        (gicd_reg.address as usize).into(),
        gicd_reg.size.unwrap_or(0x1000),
    );
    let gicc = iomap(
        (gicc_reg.address as usize).into(),
        gicc_reg.size.unwrap_or(0x1000),
    );

    Ok(FdtProbeInfo {
        hardware: Box::new(Gic::new(gicd, gicc)),
        fdt_parse_config_fn: fdt_parse_irq_config,
    })
}
