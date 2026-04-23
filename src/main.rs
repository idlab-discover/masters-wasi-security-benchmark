#[cfg(feature = "branch-blocking")]
use wasmtime_bench::instantiate_new_testcase;

#[cfg(feature = "branch-blocking")]
fn main() {
    let (call_host, mut store) = instantiate_new_testcase(Some("argument-1-all-defined.yaml"));

    let (a,) = call_host.call(&mut store, (42, 7, 13, 99)).unwrap();
    call_host.post_return(&mut store).unwrap();
    println!("Result from host call: {}", a);
}

#[cfg(not(feature = "branch-blocking"))]
fn main() {
    println!("main is only available with the branch-blocking feature");
}