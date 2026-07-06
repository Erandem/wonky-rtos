#![feature(abi_avr_interrupt)]
#![no_std]
#![no_main]

pub mod print;
pub mod task;
pub mod millis;
pub mod utils;
pub mod kernel;

use crate::kernel::get_kernel;
use crate::task::{Task, exec_context, save_context};
use crate::millis::millis;

#[unsafe(no_mangle)]
pub static mut ASM_LOG_LOC: [u8; 8] = [0; 8];

fn entry_task1() -> ! {
    println!("Task 1 started!");

    let mut last = millis();

    loop { 
        let now = millis();

        if now - last >= 100 {
            last = now;
            println!("T1: {}", now);
        }

        kyield();
    }
}

fn entry_task2() -> ! {
    println!("Task 2 started!");

    let mut last = millis();

    loop {
        let now = millis();

        if now - last >= 500 {
            last = now;
            println!("T2: {}", now);
        }

        kyield();
    }
}

fn entry_task3() -> ! {
    println!("Task 3 started!");

    let mut last = millis();

    loop {
        let now = millis();

        if now - last >= 750 {
            last = now;
            println!("T3: {}", now);
        }

        kyield();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ksched(stack_pointer: u16) -> ! {
    println!("ksched called: 0x{:04x}", stack_pointer);

    unsafe { exec_context(stack_pointer) };
    loop { }
}

#[inline(never)]
fn kyield() {
    // save_context() jumps to the scheduler after running
    unsafe { save_context() };
}

fn main() {
    let t1 = unsafe { stack_task!(entry_task1) };
    let t2 = unsafe { stack_task!(entry_task2) };
    let t3 = unsafe { stack_task!(entry_task3) };

    println!("t.sp   : 0x{:04x}", t1.stack_pointer());
    println!("t.sb   : 0x{:04x}", t1.stack_bottom() as u16);
    println!("t.ssz  : {}", t1.stack_size());
    println!("sp-sb  : {}", t1.stack_bottom() as usize - t1.stack_pointer());
    println!("t.fn   : 0x{:04x}", entry_task1 as *const () as usize);

    let kernel = get_kernel();

    unsafe { 
        kernel.add_tasks([t1, t2, t3]);
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