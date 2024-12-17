use alloc::collections::vec_deque::VecDeque;

use crate::sync::Spinlock;

use super::tcb::TaskControlBlock;

static IDLE: Spinlock<VecDeque<TaskControlBlock>> = Spinlock::new(VecDeque::new());

pub fn idle_push(tcb: TaskControlBlock) {
    IDLE.lock().push_back(tcb);
}

pub fn idle_pop() -> Option<TaskControlBlock> {
    IDLE.lock().pop_front()
}
