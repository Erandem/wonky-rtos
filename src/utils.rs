pub const TOTAL_RAM: usize = 2048;

unsafe extern "C" {
    fn utils_get_stack_pointer() -> u16;
    fn utils_get_sreg() -> u8;
    static mut __heap_start: usize;
}

pub fn get_sreg() -> u8 {
    unsafe { utils_get_sreg() }
}

pub fn get_stack_pointer() -> u16 {
    unsafe { utils_get_stack_pointer() }
}

pub fn get_heap_pointer() -> u16 {
    &raw mut __heap_start as *mut _ as u16
}

pub fn estimate_used_ram() -> u16 {
    let stack_ptr = get_stack_pointer();
    let heap_ptr = get_heap_pointer();

    TOTAL_RAM as u16 - (stack_ptr - heap_ptr)
}

/// Splits the u16 into a (lo, hi) tuple
pub const fn split_u16(value: u16) -> (u8, u8) {
    ((value & 0xFF) as u8, ((value >> 8) & 0xFF) as u8)
}