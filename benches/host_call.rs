use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use wasmtime_bench::{COMPONENT_WAT, instantiate_new_testcase};
use wasmtime_v38_0_3::{
    Engine, Store,
    component::{Component, Linker, TypedFunc},
};

const POLICY_FILES: &[&str] = &[
    "argument-1-all-defined",
    "argument-1",
    "argument-3",
    "argument-all-no-constraint",
    "argument-all",
    "function",
];

fn instantiate_v38_testcase() -> (TypedFunc<(u32, u32, u32, u32), (u32,)>, Store<()>) {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap(
            "my-host-func",
            |_store, (a, b, c, d): (u32, u32, u32, u32)| {
                Ok((a
                    .wrapping_mul(3)
                    .wrapping_add(b.wrapping_mul(5))
                    .wrapping_add(c.wrapping_mul(7))
                    .wrapping_add(d.wrapping_mul(11)),))
            },
        )
        .unwrap();

    let component = Component::new(&engine, COMPONENT_WAT).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();
    let call_host = instance
        .get_typed_func::<(u32, u32, u32, u32), (u32,)>(&mut store, "call-host")
        .unwrap();

    (call_host, store)
}

fn bench_wasmtime_host_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("guest_to_host_roundtrip");

    // Baseline benchmark from upstream Wasmtime (v38.0.3).
    let (call_host_v38, mut store_v38) = instantiate_v38_testcase();
    group.bench_with_input(
        BenchmarkId::from_parameter("v38.0.3"),
        &"v38.0.3",
        |b, _| {
            b.iter(|| {
                let args = black_box((42u32, 7u32, 13u32, 99u32));
                black_box(call_host_v38.call(&mut store_v38, args).unwrap().0);
                black_box(call_host_v38.post_return(&mut store_v38).unwrap());
            })
        },
    );

    // Fork benchmark without policy file.
    let (call_host_no_policy, mut store_no_policy) = instantiate_new_testcase(None);
    group.bench_with_input(
        BenchmarkId::from_parameter("no-policy"),
        &"no-policy",
        |b, _| {
            b.iter(|| {
                let args = black_box((42u32, 7u32, 13u32, 99u32));
                black_box(
                    call_host_no_policy
                        .call(&mut store_no_policy, args)
                        .unwrap()
                        .0,
                );
                black_box(
                    call_host_no_policy
                        .post_return(&mut store_no_policy)
                        .unwrap(),
                );
            })
        },
    );

    // Fork policy benchmarks.
    for policy_file in POLICY_FILES {
        let (call_host, mut store) =
            instantiate_new_testcase(Some(format!("{policy_file}.yaml").as_str()));
        group.bench_with_input(
            BenchmarkId::from_parameter(policy_file),
            policy_file,
            |b, _| {
                b.iter(|| {
                    let args = black_box((42u32, 7u32, 13u32, 99u32));
                    black_box(call_host.call(&mut store, args).unwrap().0);
                    black_box(call_host.post_return(&mut store).unwrap());
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_wasmtime_host_calls);
criterion_main!(benches);

