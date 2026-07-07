#![feature(abi_avr_interrupt, never_type)]
#![no_std]
#![no_main]

pub mod kernel;
pub mod millis;
pub mod print;
pub mod task;
pub mod utils;

use crate::kernel::get_kernel;
use crate::millis::millis;
use crate::task::{Task, exec_context, save_context};

#[unsafe(no_mangle)]
pub static mut ASM_LOG_LOC: [u8; 8] = [0; 8];

fn closure_wait_task(task_num: u8, wait_time: u64) -> impl FnOnce() -> ! + 'static {
    move || {
        println!("Task {} started!", task_num);

        let mut last = millis();

        loop {
            let now = millis();

            if now - last >= wait_time {
                last = now;
                println!("T{}: {}", task_num, now);
            }

            kyield();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ksched(stack_pointer: u16) -> ! {
    println!("ksched called: 0x{:04x}", stack_pointer);

    unsafe { exec_context(stack_pointer) };
    loop {}
}

#[inline(never)]
fn kyield() {
    // save_context() jumps to the scheduler after running
    unsafe { save_context() };
}

fn main() {
    let kernel = get_kernel();

    unsafe {
        kernel.add_tasks([
            stack_task!(closure_wait_task(1, 11)),
            stack_task!(closure_wait_task(2, 12)),
            stack_task!(closure_wait_task(3, 13)),
        ]);
    }

    unsafe { kernel.start() }
}

#[arduino_hal::entry]
fn entry() -> ! {
    let periphs = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(periphs);

    let serial = arduino_hal::default_serial!(periphs, pins, 230400);

    unsafe { crate::print::init(serial) };
    unsafe { crate::millis::init(periphs.TC0) };
    unsafe { crate::kernel::init() };

    main();

    println!("ERROR: Entry returned!");
    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("PANIC");

    loop {}
}

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn panic_asm() -> ! {
    println!("PANIC FROM ASM");

    #[allow(static_mut_refs)]
    for i in 0..unsafe { ASM_LOG_LOC.len() } {
        println!("0x{:02X} - {}", unsafe { ASM_LOG_LOC[i] }, i);
    }

    loop {}
}
