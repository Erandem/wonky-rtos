fn main() {
    cc::Build::new()
        .file("asm/context_switch.S")
        .file("asm/utils.S")
        .compiler("avr-gcc")
        .archiver("avr-ar")
        .flag("-mmcu=atmega328p")
        .compile("asm");

    // Tell Cargo to rerun if the assembly changes
    println!("cargo:rerun-if-changed=asm/context_switch.S");
    println!("cargo:rerun-if-changed=asm/utils.S");
}