use core::{
    alloc::Layout,
    mem::{self, ManuallyDrop},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, string::String, vec::Vec};
use log::debug;

use crate::{
    platform::PlatformImpl,
    task::queue::{finished_push, schedule},
};

use super::{queue::idle_push, TaskConfig, TaskError};

#[derive(Debug, Clone, Copy)]
pub enum TaskState {
    Idle,
    Running,
    Suspend,
    Stopped,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaskControlBlock(*mut u8);

unsafe impl Send for TaskControlBlock {}

pub struct TaskInfo {
    pub pid: usize,
    pub priority: usize,
    pub stack_size: usize,
    pub name: String,
    pub entry: Option<Box<dyn FnOnce()>>,
    pub state: TaskState,
}

impl TaskControlBlock {
    fn tcb_size(stack_size: usize) -> usize {
        size_of::<TaskInfo>() + stack_size + PlatformImpl::task_cpu_context_size()
    }

    pub(super) fn new<F>(entry: F, config: TaskConfig) -> Result<Self, TaskError>
    where
        F: FnOnce() + Send + 'static,
    {
        let entry_box = Box::new(entry);

        let buffer = NonNull::new(unsafe {
            alloc::alloc::alloc_zeroed(
                Layout::from_size_align(Self::tcb_size(config.stack_size), 16).unwrap(),
            )
        })
        .ok_or(TaskError::NoMemory)?;

        static ID_ITOR: AtomicUsize = AtomicUsize::new(0);

        let pid = ID_ITOR.fetch_add(1, Ordering::Relaxed);

        let pc;

        unsafe {
            let task_info = &mut *(buffer.as_ptr() as *mut TaskInfo);
            task_info.pid = pid;
            task_info.stack_size = config.stack_size;
            task_info.priority = config.priority;
            task_info.name = config.name;
            task_info.state = TaskState::Idle;
            task_info.entry = Some(entry_box);

            pc = task_entry as usize as _;
        }

        let mut task = Self(buffer.as_ptr());

        PlatformImpl::cpu_context_init(task.cpu_context_ptr() as _, pc, unsafe {
            task.stack().as_mut_ptr().add(config.stack_size)
        });

        Ok(task)
    }

    pub fn info(&self) -> &TaskInfo {
        unsafe { &*(self.0 as *mut TaskInfo) }
    }

    fn info_mut(&self) -> &mut TaskInfo {
        unsafe { &mut *(self.0 as *mut TaskInfo) }
    }

    pub(super) fn stack(&self) -> &mut [u8] {
        let stack_size = self.info().stack_size;
        unsafe { core::slice::from_raw_parts_mut(self.0.add(size_of::<TaskInfo>()), stack_size) }
    }

    unsafe fn drop(self) {
        let info = self.info();

        let size = Self::tcb_size(info.stack_size);
        alloc::alloc::dealloc(
            self.0 as *mut u8,
            Layout::from_size_align_unchecked(size, 16),
        );
    }

    fn cpu_context_ptr(&self) -> *mut u8 {
        unsafe {
            self.0
                .add(size_of::<TaskInfo>())
                .add(self.info().stack_size)
        }
    }

    fn addr(&self) -> *mut u8 {
        self.0 as _
    }

    pub(super) fn switch_to(&self, next: &TaskControlBlock) {
        debug!("switch {} -> {}", self.info().name, next.info().name);
        set_current(next);
        match self.info().state {
            TaskState::Stopped => finished_push(*self),
            _ => idle_push(*self),
        }

        PlatformImpl::cpu_context_switch(self.cpu_context_ptr(), next.cpu_context_ptr());
    }

    fn exit(self) {
        unsafe {
            self.drop();
        }
    }
}

pub fn current() -> TaskControlBlock {
    unsafe {
        let ptr = PlatformImpl::get_current_tcb_addr();
        TaskControlBlock(ptr as _)
    }
}

pub fn set_current(tcb: &TaskControlBlock) {
    unsafe {
        PlatformImpl::set_current_tcb_addr(tcb.addr());
    }
}

extern "C" fn task_entry() -> ! {
    let task = current();
    unsafe {
        let task_mut = task.info_mut();
        if let Some(entry) = task_mut.entry.take() {
            entry();
            task_mut.state = TaskState::Stopped;
        }
    }
    schedule();
    unreachable!("task exited!");
}
