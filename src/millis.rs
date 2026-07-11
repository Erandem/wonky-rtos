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

// 16 MHz / 1 prescaler / 1000 us per ms = 16,000 ticks per ms
pub const TICKS_PER_MS: u16 = (16_000_000 / 1 / 1000) as u16;
pub const PERIOD_MS: u8 = 1;
pub const OCR1A_TOP: u16 = TICKS_PER_MS - 1;

static mut TC1_COUNTER: u32 = 0;

pub unsafe fn init_tc1(tc1: TC1) {
    tc1.tccr1a().write(|w| unsafe { w.wgm1().bits(0b00) });
    tc1.tccr1b().write(|w| unsafe { w.wgm1().bits(0b01) }.cs1().direct());
    tc1.ocr1a().write(|w| w.set(OCR1A_TOP));

    // Enable Output A Match interrupt
    tc1.timsk1().write(|w| w.ocie1a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    unsafe { TC1_COUNTER = TC1_COUNTER.unchecked_add(1) };
}

#[inline(never)]
pub fn millis_tc1() -> u64 {
    avr_device::interrupt::free(|_| {
        unsafe { TC1_COUNTER as u64 }
    })
}

pub fn micros() -> u64 {
    avr_device::interrupt::free(|_| {
        // SAFETY: We are only reading from the timer, not changing anything, therefore it's safe to steal it here.
        let tc1 = unsafe { TC1::steal() };

        // Read raw tick and ms count first
        let ticks = tc1.tcnt1().read().bits();
        let ms = unsafe { TC1_COUNTER };

        // ...then check if the interrupt flag is set.
        // If global interrupts are masked, the overflow timer may have hit and the timer
        // has reset, but the global counter hasn't been incremented yet. This means the
        // ticks we've read are correct, but we are off by 1 overflow.
        let pre_interrupt_offset = if tc1.tifr1().read().ocf1a().bit_is_set() {
            PERIOD_MS as u64
        } else {
            0
        };

        (ms as u64).wrapping_mul(1000) + (ticks as u64 / 16) + pre_interrupt_offset
    })
}

#[inline(always)]
pub fn millis() -> u64 {
    millis_tc1()
}

#[inline(never)]
pub fn millis_tc0() -> u64 {
    avr_device::interrupt::free(|_| {
        unsafe { MILLIS_COUNTER_VOLATILE }
    })
}