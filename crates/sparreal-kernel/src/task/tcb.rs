use core::{
    alloc::Layout,
    mem::ManuallyDrop,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, vec::Vec};

use crate::platform::PlatformImpl;

use super::TaskConfig;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaskControlBlock(usize);

pub struct TaskInfo {
    pub pid: usize,
    pub priority: usize,
    pub stack_size: usize,
}

impl TaskControlBlock {
    fn tcb_size(stack_size: usize) -> usize {
        size_of::<TaskInfo>() + stack_size + PlatformImpl::task_cpu_context_size()
    }

    pub(super) fn new(config: TaskConfig) -> Option<Self> {
        let buffer = NonNull::new(unsafe {
            alloc::alloc::alloc_zeroed(
                Layout::from_size_align(Self::tcb_size(config.stack_size), 16).unwrap(),
            )
        })?;

        static ID_ITOR: AtomicUsize = AtomicUsize::new(0);

        let pid = ID_ITOR.fetch_add(1, Ordering::Relaxed);

        unsafe {
            let task_info = &mut *(buffer.as_ptr() as *mut TaskInfo);
            task_info.pid = pid;
            task_info.stack_size = config.stack_size;
            task_info.priority = config.priority;
        }

        Some(Self(buffer.as_ptr() as usize))
    }

    pub fn info(&self) -> &TaskInfo {
        unsafe { &*(self.0 as *mut TaskInfo) }
    }

    pub(super) unsafe fn stack(&self) -> &mut [u8] {
        let stack_size = self.info().stack_size;
        core::slice::from_raw_parts_mut((self.0 + size_of::<TaskInfo>()) as *mut u8, stack_size)
    }

    unsafe fn drop(self) {
        let info = self.info();
        let size = Self::tcb_size(info.stack_size);
        alloc::alloc::dealloc(
            self.0 as *mut u8,
            Layout::from_size_align_unchecked(size, 16),
        );
    }
}

pub fn current<'a>() -> &'a TaskControlBlock {
    unsafe {
        let ptr = PlatformImpl::get_current_tcb_addr();
        &*(ptr as *const TaskControlBlock)
    }
}

pub fn set_current(tcb: &TaskControlBlock) {
    unsafe {
        PlatformImpl::set_current_tcb_addr(tcb as *const _ as usize);
    }
}
