use std::hint::black_box;
extern crate criterion;
use criterion::{criterion_group, criterion_main, Criterion};
extern crate wasmtime;
use wasmtime::*;
use wasmtime::component::{Component, Linker};

fn bench_wasmtime_host_calls(c: &mut Criterion) {
    // 1. Setup the Wasmtime Engine and Store
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    // 2. Define your idempotent host function
    // This simple function takes an i32, adds 1, and returns it.
    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap("my-host-func", |_store, (param,): (u32,)| Ok((param + 1,)))
        .unwrap();

    // 3. Define the Guest Wasm module (using WAT for simplicity)
    // It imports the host function and exports a function that calls it.
    let wat = r#"
        (component
            (import "bench:host/api"
                (instance $host
                    (export "my-host-func" (func $my-host-func (param "x" u32) (result u32)))
                )
            )

            (core module $m
                (import "" "my-host-func" (func $my-host-func (param i32) (result i32)))
                (func (export "call-host") (param i32) (result i32)
                    local.get 0
                    call $my-host-func
                )
            )

            (core func $my-host-func (canon lower (func $host "my-host-func")))
            (core instance $i (instantiate $m
                (with "" (instance
                    (export "my-host-func" (func $my-host-func))
                ))
            ))

            (func (export "call-host") (param "x" u32) (result u32)
                (canon lift (core func $i "call-host"))
            )
        )
    "#;

    // 4. Compile and Instantiate
    let component = Component::new(&engine, wat).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();

    // 5. Call the component export that forwards to the lowered host function.
    let call_host = instance
        .get_typed_func::<(u32,), (u32,)>(&mut store, "call-host")
        .unwrap();

    // 6. Run the Criterion Benchmark
    c.bench_function("guest_to_host_roundtrip", |b| {
        b.iter(|| {
            // black_box prevents the compiler from optimizing away the call.
            black_box(call_host.call(&mut store, black_box((42,))).unwrap().0);
            black_box(call_host.post_return(&mut store).unwrap());
        })
    });
}

criterion_group!(benches, bench_wasmtime_host_calls);
criterion_main!(benches);