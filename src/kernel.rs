use crate::{stack_task, task::Task};

use core::cell::UnsafeCell;

use heapless::Vec;
use once_cell::sync::OnceCell;

pub static KERNEL: OnceCell<Kernel> = OnceCell::new();

pub struct Kernel {
    ks: UnsafeCell<KernelState>,
}

impl Kernel {
    unsafe fn new() -> Kernel {
        let ks = KernelState {
            current_task: 0,
            tasks: Vec::new(),
            scheduler_task: unsafe { stack_task!(scheduler_task) },
        };

        Kernel { ks: UnsafeCell::new(ks) }
    }

    /// # Safety
    /// 
    /// This function assumes that
    /// 1. Interrupts are disabled
    /// 2. It is the only interface into the kernel
    pub unsafe fn scheduler_tick(&self, stack_pointer: u16) -> ! {
        let ks = unsafe { self.get_ks() };

        ks.tasks[ks.current_task].set_stack_pointer(stack_pointer as usize);

        unsafe { stack_task!(scheduler_task).exec() }
    }

    pub unsafe fn add_task(&self, task: Task) {
        let ks = unsafe { self.get_ks() };

        ks.tasks.push(task).unwrap();
    }

    pub unsafe fn add_tasks(&self, tasks: impl IntoIterator<Item = Task>) {
        let ks = unsafe { self.get_ks() };

        for task in tasks {
            ks.tasks.push(task).unwrap();
        }
    }

    pub unsafe fn start(&self) -> ! {
        let ks = unsafe { self.get_ks() };

        unsafe { ks.scheduler_task.exec() };
    }

    unsafe fn get_ks(&self) -> &mut KernelState {
        unsafe { &mut *self.ks.get() }
    }
}

unsafe impl Send for Kernel {}

unsafe impl Sync for Kernel {}

struct KernelState {
    current_task: usize,
    tasks: Vec<Task, 4>,
    scheduler_task: Task,
}

impl KernelState {

}

unsafe impl Send for KernelState {}

unsafe impl Sync for KernelState {}

pub unsafe fn init() {
    KERNEL.get_or_init(|| {
        unsafe { Kernel::new() }
    });
}

/// Returns a reference to the Kernel or panics.
pub fn get_kernel() -> &'static Kernel {
    KERNEL.get().unwrap()
}

fn scheduler_task() -> ! {
    // Interrupts are enabled by default when a task is created.
    // We want to cancel this, as the scheduler task does not respond to interrupts.
    avr_device::interrupt::disable();

    let kernel = get_kernel();

    loop {
        avr_device::interrupt::disable();

        //println!("Scheduler Task Running!");

        let ks = unsafe { kernel.get_ks() };
        ks.current_task = ks.current_task.wrapping_add(1) % ks.tasks.len();

        unsafe { ks.tasks[ks.current_task].exec() };
    }
}

mod abi {
    use super::get_kernel;

    #[unsafe(export_name = "kernel_scheduler_tick")]
    pub unsafe extern "C" fn scheduler_tick(stack_pointer: u16) -> ! {
        unsafe { get_kernel().scheduler_tick(stack_pointer) }
    }
}