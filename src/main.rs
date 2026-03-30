use wasmtime_bench::{instantiate_new_testcase};

fn main() {
    let (call_host, mut store) = instantiate_new_testcase(Some("argument-1-all-defined.yaml"));

    let (a,) = call_host.call(&mut store, (42, 7, 13, 99)).unwrap();
    call_host.post_return(&mut store).unwrap();
    println!("Result from host call: {}", a);
}