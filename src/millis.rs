use arduino_hal::pac::TC0;

const PRESCALER: u32 = 64;
const TIMER_COUNTS: u32 = 250;

static mut MILLIS_COUNTER_VOLATILE: u64 = 0;

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
    unsafe { MILLIS_COUNTER_VOLATILE = MILLIS_COUNTER_VOLATILE.unchecked_add(1) };
}

#[inline(never)]
pub fn millis() -> u64 {
    avr_device::interrupt::free(|_| {
        unsafe { MILLIS_COUNTER_VOLATILE }
    })
}