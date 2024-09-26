use core::{
    fmt::Display,
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut, NonNull},
};

use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use device_tree::{get_device_tree, FDTExtend};
use driver_interface::timer;
use flat_device_tree::{node::FdtNode, Fdt};
use irq::init_irq;
use log::{debug, error, info};

use crate::{
    stdout::{set_stdout, UartWrite},
    sync::RwLock,
};

mod container;
pub mod device_tree;
mod irq;

pub use container::*;

pub use driver_interface::uart;
pub use irq::DriverIrqChip;

pub async fn init() {
    init_irq().await;
    init_stdout().await;
    info!("Stdout ok!");

    init_all().await;
}

pub type DriverArc<T> = Arc<RwLock<T>>;
pub type DriverWeak<T> = Weak<RwLock<T>>;

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DriverId(String);

impl From<&str> for DriverId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Display for DriverId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct DriverDescriptor {
    pub id: DriverId,
    pub name: String,
}

#[derive(Clone)]
pub(crate) struct DriverCommon<T> {
    pub desc: DriverDescriptor,
    pub spec: DriverArc<T>,
}
impl<T> DriverCommon<T> {
    pub fn new<N: ToString>(id: DriverId, name: N, spec: T) -> DriverCommon<T> {
        Self {
            desc: DriverDescriptor {
                id,
                name: name.to_string(),
            },
            spec: Arc::new(RwLock::new(spec)),
        }
    }
}

impl Display for DriverDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Driver({}) {}", self.id.0, self.name)
    }
}

#[macro_export(local_inner_macros)]
macro_rules! struct_driver {
    ($name:ident, $spec:ty) => {
        #[derive(Clone)]
        pub struct $name {
            pub desc: $crate::driver::DriverDescriptor,
            pub spec: $crate::driver::DriverArc<$spec>,
        }
        impl From<$crate::driver::DriverCommon<$spec>> for $name {
            fn from(value: $crate::driver::DriverCommon<$spec>) -> Self {
                Self {
                    desc: value.desc,
                    spec: value.spec,
                }
            }
        }
    };
}

pub unsafe fn move_dtb(src: *const u8, mut dst: NonNull<u8>) -> Option<&'static [u8]> {
    let fdt = Fdt::from_ptr(src).ok()?;
    let size = fdt.total_size();
    let dest = &mut *slice_from_raw_parts_mut(dst.as_mut(), size);
    let src = &*slice_from_raw_parts(src, size);
    dest.copy_from_slice(src);
    Some(dest)
}

async fn init_stdout() -> Option<()> {
    let fdt = get_device_tree()?;
    let chosen = fdt.chosen().ok()?;
    let stdout = chosen.stdout()?;
    let node = stdout.node();
    let caps = node.compatible()?.all().collect::<Vec<_>>();

    let register = register_by_compatible(&caps)?;

    probe(node.name.into(), node.probe_config(), register).await?;

    let driver = uart_by_id(node.name.into())?;

    let stdout = UartWrite::new(&driver.spec);

    set_stdout(stdout);

    Some(())
}

async fn init_all() {
    if let Some(fdt) = get_device_tree() {
        for node in fdt
            .all_nodes()
            .filter(|node| !node.name.contains("memory@"))
        {
            probe_by_node(node).await;
        }
    }
}

struct_driver!(DriverUart, uart::BoxDriver);
struct_driver!(DriverTimer, timer::BoxDriver);
