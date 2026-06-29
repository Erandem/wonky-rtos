use arduino_hal::pac::TC0;
use portable_atomic::{AtomicU32, Ordering};

const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;

static MILLIS_COUNTER: AtomicU32 = AtomicU32::new(0);

pub unsafe fn init(tc0: TC0) {
    tc0.tccr0a().write(|w| w.wgm0().ctc());
    tc0.ocr0a().write(|w| w.set(TIMER_COUNTS as u8));

    tc0.tccr0b().write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });

    tc0.timsk0().write(|w| w.ocie0a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    MILLIS_COUNTER.add(1, Ordering::Relaxed);
}

pub fn millis() -> u32 {
    MILLIS_COUNTER.load(Ordering::Relaxed)
}