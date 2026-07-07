use crate::utils::split_u16;

pub const DEFAULT_STACK_SIZE: usize = 256;
pub const STACK_GUARD: u8 = 0xE1;

#[macro_export]
macro_rules! stack_task {
    ($func:expr) => {
        stack_task!($func, stack_size: $crate::task::DEFAULT_STACK_SIZE)
    };

    ($func:expr, stack_size: $stack_size:expr) => {
        stack_task!($func, stack_size: $stack_size, stack_guard: $crate::task::STACK_GUARD)
    };

    ($func:expr, stack_size: $stack_size:expr, stack_guard: $stack_guard:expr) => {
        {
            static mut TASK_STACK: [u8; $stack_size] = [$stack_guard; $stack_size];

            #[allow(static_mut_refs)]
            Task::new_closure(&mut TASK_STACK, $func)
        }
    };
}

unsafe extern "C" {
    /// Saves registers and SREG, and returns the stack pointer
    pub fn save_context() -> u16;
    pub fn exec_context(stack_pointer: u16);
}

// Need repr(C) so we can guarantee the memory layout for saving and storing the stack pointer
#[repr(C)]
#[derive(Debug)]
pub struct Task {
    stack_pointer: usize,
    stack_bottom: *mut u8,
    stack_size: usize,
}

impl Task {
    pub fn new(stack: &'static mut [u8], entry_fn: fn() -> !) -> Task {
        // AVR uses post-descent, so bottom is the last valid memory address
        let stack_bottom = unsafe { stack.as_mut_ptr().add(stack.len() - 1) };
        let stack_pointer = stack_bottom as usize;

        let mut task = Task {
            stack_pointer,
            stack_bottom,
            stack_size: stack.len(),
        };

        // Push entry function address to the end of the stack
        let entry_addr = entry_fn as usize;
        let entry_addr_low = (entry_addr & 0xFF) as u8;
        let entry_addr_high = ((entry_addr >> 8) & 0xFF) as u8;

        unsafe {
            task.push(entry_addr_low);
            task.push(entry_addr_high);
        }
 
        // Push empty registers to the stack
        unsafe { task.push_registers(Registers::new()) }

        // Push SREG
        unsafe { task.push(0x80) } // SREG with I-bit set

        task
    }

    pub unsafe fn new_closure<F>(stack: &'static mut [u8], closure: F) -> Task
    where
        F: FnOnce() -> ! + 'static,
    {
        let closure_size = core::mem::size_of::<F>();
        let closure_align = core::mem::align_of::<F>();

        let stack_start = stack.as_mut_ptr() as usize;
        let closure_addr = (stack_start + closure_align - 1) & !(closure_align - 1);
        let (closure_addr_lo, closure_addr_hi) = split_u16(closure_addr as u16);

        let stack_bottom = unsafe { stack.as_mut_ptr().add(stack.len() - 1) };

        assert!(
            closure_addr + closure_size <= stack_bottom as usize,
            "stack too small to hold closure!",
        );

        // Write closure address for trampoline
        unsafe { core::ptr::write(closure_addr as *mut F, closure) };

        let stack_pointer = stack_bottom as usize;
        let mut task = Task {
            stack_pointer,
            stack_bottom,
            stack_size: stack.len(),
        };

        // Push entry function address to the end of the stack
        unsafe { task.push_addr(trampoline::<F> as *const () as usize) }

        // Then push the registers
        unsafe {
            task.push_registers(
                Registers::new()
                    .set_reg(24, closure_addr_lo)
                    .set_reg(25, closure_addr_hi),
            )
        };

        // Then push SREG with I-bit set
        unsafe { task.push(0x80) }; // SREG with I-bit set
        task
    }

    pub unsafe fn exec(&self) -> ! {
        unsafe { exec_context(self.stack_pointer as u16) }

        panic!("exec_context returned");
    }

    // Pushes a value to the stack
    unsafe fn push(&mut self, value: u8) {
        unsafe { *(self.stack_pointer as *mut u8) = value };
        self.stack_pointer -= 1;
    }

    unsafe fn push_addr(&mut self, addr: usize) {
        let (addr_lo, addr_high) = split_u16(addr as u16);

        unsafe {
            self.push(addr_lo);
            self.push(addr_high);
        }
    }

    unsafe fn push_registers(&mut self, registers: Registers) {
        for &reg in registers.get() {
            unsafe { self.push(reg) };
        }
    }

    pub fn set_stack_pointer(&mut self, stack_pointer: usize) {
        self.stack_pointer = stack_pointer
    }

    pub fn stack_pointer(&self) -> usize {
        self.stack_pointer
    }

    pub fn stack_bottom(&self) -> *mut u8 {
        self.stack_bottom
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }
}

#[cfg(target_arch = "avr")]
unsafe impl Send for Task {}

#[cfg(target_arch = "avr")]
unsafe impl Sync for Task {}

pub struct Registers {
    regs: [u8; 32],
}

impl Registers {
    pub const fn new() -> Registers {
        Registers { regs: [0; 32] }
    }

    pub const fn set_reg(mut self, reg: usize, value: u8) -> Self {
        self.regs[reg] = value;

        self
    }

    pub const fn get(&self) -> &[u8; 32] {
        &self.regs
    }
}

unsafe extern "C" fn trampoline<F>(closure_ptr: *mut F) -> !
where
    F: FnOnce() -> ! + 'static,
{
    let closure = unsafe { core::ptr::read(closure_ptr) };
    closure()
}
