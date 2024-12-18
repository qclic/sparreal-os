use alloc::collections::vec_deque::VecDeque;

use crate::{platform::PlatformImpl, sync::Spinlock};

use super::tcb::{current, TaskControlBlock};

static IDLE: Spinlock<VecDeque<TaskControlBlock>> = Spinlock::new(VecDeque::new());
static FINISHED: Spinlock<VecDeque<TaskControlBlock>> = Spinlock::new(VecDeque::new());

pub fn idle_push(tcb: TaskControlBlock) {
    IDLE.lock().push_back(tcb);
}

pub fn idle_pop() -> Option<TaskControlBlock> {
    IDLE.lock().pop_front()
}

pub fn finished_push(tcb: TaskControlBlock) {
    FINISHED.lock().push_back(tcb);
}

pub fn schedule() {
    let idle = idle_pop();
    if let Some(idle) = idle {
        current().switch_to(&idle);
    } else {
        loop {
            unsafe { PlatformImpl::wait_for_interrupt() };
        }
    }
}
