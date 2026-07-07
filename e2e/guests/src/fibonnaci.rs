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

fn main() {
    std::hint::black_box(fibonacci(40));
}