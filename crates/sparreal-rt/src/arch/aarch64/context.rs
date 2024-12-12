use core::arch::naked_asm;

/// Saved registers when a trap (exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    /// General-purpose registers (X0..X30).
    pub x: [u64; 31],
    /// User Stack Pointer (SP_EL0).
    pub usp: u64,
    /// Exception Link Register (ELR_EL1).
    pub elr: u64,
    /// Saved Process Status Register (SPSR_EL1).
    pub spsr: u64,
}

/// FP & SIMD registers.
#[cfg(hard_float)]
#[repr(C, align(16))]
#[derive(Debug, Default)]
pub struct FpState {
    /// 128-bit SIMD & FP registers (V0..V31)
    pub regs: [u128; 32],
    /// Floating-point Control Register (FPCR)
    pub fpcr: u64,
    /// Floating-point Status Register (FPSR)
    pub fpsr: u64,
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug)]
pub struct CpuContext {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub fp: u64,
    pub sp: u64,
    pub pc: u64,
    #[cfg(hard_float)]
    pub fp_state: FpState,
}

const TASK_CONTEXT_SIZE: usize = size_of::<CpuContext>();

impl CpuContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe { context_switch(self, next_ctx) }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut CpuContext, _next_task: &CpuContext) {
    naked_asm!(
        "
        mov  x9, sp
        stp  x19, x20, [x0], #16
        stp  x21, x22, [x0], #16
        stp  x23, x24, [x0], #16
        stp  x25, x26, [x0], #16
        stp  x27, x28, [x0], #16
        stp  x29, x9,  [x0], #16
        str  lr,       [x0]",
        #[cfg(hard_float)]
        "
        mrs     x9, fpcr
        mrs     x10, fpsr
        stp     q0,  q1, [x0], #32
        stp     q2,  q3, [x0], #32
        stp     q4,  q5, [x0], #32
        stp     q6,  q7, [x0], #32
        stp     q8,  q9, [x0], #32
        stp     q10, q11, [x0], #32
        stp     q12, q13, [x0], #32
        stp     q14, q15, [x0], #32
        stp     q16, q17, [x0], #32
        stp     q18, q19, [x0], #32
        stp     q20, q21, [x0], #32
        stp     q22, q23, [x0], #32
        stp     q24, q25, [x0], #32
        stp     q26, q27, [x0], #32
        stp     q28, q29, [x0], #32
        stp     q30, q31, [x0], #32
        stp     x9,  x10, [x0]
        ",
        "ldp    x19, x20, [x1], #16",
        "ldp    x21, x22, [x1], #16",
        "ldp    x23, x24, [x1], #16",
        "ldp    x25, x26, [x1], #16",
        "ldp    x27, x28, [x1], #16",
        "ldp    x29, x9,  [x1], #16",
        "ldr    lr,       [x1]",
        "mov    sp,  x9",
        #[cfg(hard_float)]
        "
        ldp    q0,  q1,  [x1], #16
        ldp    q2,  q3,  [x1], #16
        ldp    q4,  q5,  [x1], #16
        ldp    q6,  q7,  [x1], #16
        ldp    q8,  q9,  [x1], #16
        ldp    q10, q11, [x1], #16
        ldp    q12, q13, [x1], #16
        ldp    q14, q15, [x1], #16
        ldp    q16, q17, [x1], #16
        ldp    q18, q19, [x1], #16
        ldp    q20, q21, [x1], #16
        ldp    q22, q23, [x1], #16
        ldp    q24, q25, [x1], #16
        ldp    q26, q27, [x1], #16
        ldp    q28, q29, [x1], #16
        ldp    q30, q31, [x1], #16
        ldp    x8,  x9,  [x1], #16
        msr     fpcr, x9
        msr     fpsr, x10
        ",
        "ret",
    )
}
