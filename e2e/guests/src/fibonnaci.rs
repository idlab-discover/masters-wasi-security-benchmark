#![no_std]
#![no_main]

/// Fibonacci benchmark - CPU-intensive recursive workload
/// 
/// This is the canonical CPU benchmark for Wasm runtimes.
/// Fibonacci(40) takes ~1-2 seconds on modern hardware,
/// providing enough data for statistical analysis.

/// Recursive Fibonacci implementation
/// Intentionally not optimized - we want to measure function call overhead
fn fibonacci(n: i32) -> i64 {
    if n <= 1 {
        return n as i64;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn __wasi_init_tp() {}

#[no_mangle]
pub extern "C" fn __main_void() -> i32 {
    core::hint::black_box(fibonacci(40));
    0
}

#[no_mangle]
pub extern "C" fn __wasm_call_dtors() {}