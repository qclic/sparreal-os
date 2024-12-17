mod queue;
mod tcb;

use alloc::string::String;
pub use tcb::TaskControlBlock;

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub name: String,
    pub priority: usize,
    pub stack_size: usize,
}

pub fn spawn_with_config<F>(f: F, config: TaskConfig)
where
    F: FnOnce() + Send + 'static,
{
}
