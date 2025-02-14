pub use driver_interface::timer::Hardware;

pub type OnProbeFdt = fn(node: super::FdtInfo<'_>) -> Hardware;
