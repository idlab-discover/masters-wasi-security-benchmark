#![no_std]
#![no_main]

// No-op guest program to measure baseline initialization and startup overhead.

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn __wasi_init_tp() {}

#[no_mangle]
pub extern "C" fn __main_void() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn __wasm_call_dtors() {}
