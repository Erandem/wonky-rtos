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
        for _ in 0..32 {
            unsafe { task.push(0) };
        }

        // Push SREG
        unsafe { task.push(0x80) } // SREG with I-bit set

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

    pub fn set_stack_pointer(&mut self, stack_pointer: usize) { self.stack_pointer = stack_pointer }

    pub fn stack_pointer(&self) -> usize { self.stack_pointer }

    pub fn stack_bottom(&self) -> *mut u8 { self.stack_bottom }

    pub fn stack_size(&self) -> usize { self.stack_size }
}

#[cfg(target_arch = "avr")]
unsafe impl Send for Task {}

#[cfg(target_arch = "avr")]
unsafe impl Sync for Task {}