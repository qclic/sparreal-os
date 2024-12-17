mod queue;
mod tcb;

use core::mem;

use alloc::{boxed::Box, string::String};
use tcb::set_current;
pub use tcb::TaskControlBlock;

use crate::platform::PlatformImpl;

#[derive(Debug, Clone)]
pub enum TaskError {
    NoMemory,
}

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub name: String,
    pub priority: usize,
    pub stack_size: usize,
}

pub fn spawn_with_config<F>(f: F, config: TaskConfig) -> Result<(), TaskError>
where
    F: FnOnce() + Send + 'static,
{
    let task = TaskControlBlock::new(f, config)?;

    Ok(())
}

pub fn init() {
    let task = TaskControlBlock::new(
        || {},
        TaskConfig {
            name: "Main".into(),
            priority: 0,
            stack_size: 0,
        },
    )
    .unwrap();
    set_current(task);
}
