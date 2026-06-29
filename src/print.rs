use core::{cell::UnsafeCell, mem::MaybeUninit};

type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
pub static CONSOLE: SharedConsole = SharedConsole::new();

#[macro_export]
macro_rules! print {
    ($($t:tt)*) => {
        avr_device::interrupt::free(|_| {
            let console_uninit = unsafe { &mut *$crate::print::CONSOLE.console.get() };
            let console = unsafe { console_uninit.assume_init_mut() };
            let _ = ufmt::uwrite!(console, $($t)*);
        })
    };
}

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => {
        avr_device::interrupt::free(|_| {
            let console_uninit = unsafe { &mut *$crate::print::CONSOLE.console.get() };
            let console = unsafe { console_uninit.assume_init_mut() };
            let _ = ufmt::uwriteln!(console, $($t)*);
        })
    };
}

pub struct SharedConsole {
    pub console: UnsafeCell<MaybeUninit<Console>>,
}

impl SharedConsole {
    const fn new() -> SharedConsole {
        SharedConsole { console: UnsafeCell::new(MaybeUninit::uninit()) }
    }

    unsafe fn init(&self, console: Console) {
        avr_device::interrupt::free(|_| {
            // SAFETY: We pass safety guarantees onto the caller
            unsafe { (*CONSOLE.console.get()).write(console) };
        })
    }
}

pub unsafe fn init(console: Console) {
    // SAFETY: We pass safety requirements onto the caller
    unsafe { CONSOLE.init(console) };
}

unsafe impl Send for SharedConsole {}

unsafe impl Sync for SharedConsole {}