use std::hint::black_box;
#[cfg(feature = "branch-opa-local")]
use std::path::Path;
#[cfg(feature = "branch-blocking")]
use std::sync::Arc;
use std::time::Duration;

use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
use wasmtime_bench::COMPONENT_WAT;

#[cfg(feature = "branch-blocking")]
use wasmtime_blocking_rules_policy_file as wasmtime_selected;
#[cfg(feature = "branch-opa-local")]
use wasmtime_opa_local as wasmtime_selected;
#[cfg(feature = "branch-opa-test")]
use wasmtime_opa_test as wasmtime_selected;
#[cfg(feature = "branch-upstream")]
use wasmtime_v38_0_3 as wasmtime_selected;

#[cfg(not(any(
    feature = "branch-blocking",
    feature = "branch-opa-local",
    feature = "branch-opa-test",
    feature = "branch-upstream"
)))]
compile_error!(
    "Enable one branch feature: branch-blocking, branch-opa-local, branch-opa-test, or branch-upstream"
);

#[cfg(any(
    all(feature = "branch-blocking", feature = "branch-opa-local"),
    all(feature = "branch-blocking", feature = "branch-opa-test"),
    all(feature = "branch-blocking", feature = "branch-upstream"),
    all(feature = "branch-opa-local", feature = "branch-opa-test"),
    all(feature = "branch-opa-local", feature = "branch-upstream"),
    all(feature = "branch-opa-test", feature = "branch-upstream")
))]
compile_error!("Enable exactly one branch feature at a time.");

#[derive(Clone, Copy)]
enum BranchPolicyConfig {
    #[cfg(feature = "branch-blocking")]
    BlockingRulesPolicyFile { policy_file: &'static str },
    #[cfg(feature = "branch-opa-local")]
    OpaLocal {
        rules_file: &'static str,
        data_file: &'static str,
    },
    #[cfg(feature = "branch-opa-test")]
    OpaTest { opa_url: &'static str },
    #[cfg(feature = "branch-upstream")]
    Upstream,
}

#[derive(Clone, Copy)]
struct BranchCase {
    cli_args: &'static str,
    #[allow(dead_code)]
    config: BranchPolicyConfig,
}

#[cfg(feature = "branch-blocking")]
const ACTIVE_CASES: &[BranchCase; 7] = &[
    BranchCase {
        cli_args: "allow",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "allow.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-1-all-defined",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "argument-1-all-defined.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-1",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "argument-1.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-3",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "argument-3.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-all-no-constraint",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "argument-all-no-constraint.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-all",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "argument-all.yaml",
        },
    },
    BranchCase {
        cli_args: "function",
        config: BranchPolicyConfig::BlockingRulesPolicyFile {
            policy_file: "function.yaml",
        },
    },
];

#[cfg(feature = "branch-opa-local")]
const ACTIVE_CASES: &[BranchCase; 7] = &[
    BranchCase {
        cli_args: "allow",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "allow.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-1-all-defined",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "argument-1-all-defined.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-1",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "argument-1.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-3",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "argument-3.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-all-no-constraint",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "argument-all-no-constraint.yaml",
        },
    },
    BranchCase {
        cli_args: "argument-all",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "argument-all.yaml",
        },
    },
    BranchCase {
        cli_args: "function",
        config: BranchPolicyConfig::OpaLocal {
            rules_file: "rules.rego",
            data_file: "function.yaml",
        },
    },
];

#[cfg(feature = "branch-opa-test")]
const ACTIVE_CASES: &[BranchCase; 7] = &[
    BranchCase {
        cli_args: "allow",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8181/v1/data/component/host_function/allow",
        },
    },
    BranchCase {
        cli_args: "argument-1-all-defined",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8182/v1/data/component/host_function/allow",
        },
    },
    BranchCase {
        cli_args: "argument-1",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8183/v1/data/component/host_function/allow",
        },
    },
    BranchCase {
        cli_args: "argument-3",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8184/v1/data/component/host_function/allow",
        },
    },
    BranchCase {
        cli_args: "argument-all-no-constraint",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8185/v1/data/component/host_function/allow",
        },
    },
    BranchCase {
        cli_args: "argument-all",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8186/v1/data/component/host_function/allow",
        },
    },
    BranchCase {
        cli_args: "function",
        config: BranchPolicyConfig::OpaTest {
            opa_url: "http://localhost:8187/v1/data/component/host_function/allow",
        },
    },
];

#[cfg(feature = "branch-upstream")]
const ACTIVE_CASES: &[BranchCase] = &[BranchCase {
    cli_args: "allow",
    config: BranchPolicyConfig::Upstream,
}];

fn case_arg_display(case: &BranchCase) -> String {
    case.cli_args.to_string()
}

fn host_impl(a: u32, b: u32, c: u32, d: u32) -> u32 {
    a.wrapping_mul(3)
        .wrapping_add(b.wrapping_mul(5))
        .wrapping_add(c.wrapping_mul(7))
        .wrapping_add(d.wrapping_mul(11))
}

#[cfg(feature = "branch-upstream")]
fn v38_case(group: &mut BenchmarkGroup<'_, WallTime>, case: &BranchCase) {
    let engine = wasmtime_selected::Engine::default();
    let mut store = wasmtime_selected::Store::new(&engine, ());
    let mut linker = wasmtime_selected::component::Linker::new(&engine);

    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap(
            "my-host-func",
            |_store, (a, b, c, d): (u32, u32, u32, u32)| Ok((host_impl(a, b, c, d),)),
        )
        .unwrap();

    let component = wasmtime_selected::component::Component::new(&engine, COMPONENT_WAT).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();
    let call_host = instance
        .get_typed_func::<(u32, u32, u32, u32), (u32,)>(&mut store, "call-host")
        .unwrap();

    group.bench_with_input(
        BenchmarkId::new("v38.0.3 (baseline)", case_arg_display(case)),
        case,
        |b, _| {
            b.iter(|| {
                let args = black_box((42u32, 7u32, 13u32, 99u32));
                black_box(call_host.call(&mut store, args).unwrap().0);
                black_box(call_host.post_return(&mut store).unwrap());
            })
        },
    );
}

#[cfg(feature = "branch-blocking")]
fn blocking_branch_case(group: &mut BenchmarkGroup<'_, WallTime>, case: &BranchCase) {
    let BranchPolicyConfig::BlockingRulesPolicyFile { policy_file } = case.config;

    let policy_str = std::fs::read_to_string(policy_file).unwrap_or_else(|err| {
        panic!("failed to read policy file '{policy_file}': {err}");
    });
    let policy = serde_yaml::from_str::<wasmtime_selected::component::WasmPolicy>(&policy_str)
        .unwrap_or_else(|err| panic!("failed to parse policy file '{policy_file}': {err}"));

    let engine = wasmtime_selected::Engine::default();
    let mut store = wasmtime_selected::Store::new(&engine, ());
    let mut linker =
        wasmtime_selected::component::Linker::new_with_policy(&engine, Arc::new(policy));

    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap(
            "my-host-func",
            |_store, (a, b, c, d): (u32, u32, u32, u32)| Ok((host_impl(a, b, c, d),)),
        )
        .unwrap();

    let component = wasmtime_selected::component::Component::new(&engine, COMPONENT_WAT).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();
    let call_host = instance
        .get_typed_func::<(u32, u32, u32, u32), (u32,)>(&mut store, "call-host")
        .unwrap();

    group.bench_with_input(
        BenchmarkId::new("blocking", case_arg_display(case)),
        case,
        |b, _| {
            b.iter(|| {
                let args = black_box((42u32, 7u32, 13u32, 99u32));
                black_box(call_host.call(&mut store, args).unwrap().0);
                black_box(call_host.post_return(&mut store).unwrap());
            })
        },
    );
}

#[cfg(feature = "branch-opa-local")]
fn opa_local_branch_case(group: &mut BenchmarkGroup<'_, WallTime>, case: &BranchCase) {
    let BranchPolicyConfig::OpaLocal {
        rules_file,
        data_file,
    } = case.config;

    let mut config = wasmtime_selected::Config::new();
    config
        .wasm_policy(Path::new(rules_file), Some(Path::new(data_file)))
        .unwrap_or_else(|err| {
            panic!("failed to configure OPA_local from '{rules_file}' and '{data_file}': {err}")
        });

    let engine = wasmtime_selected::Engine::new(&config).unwrap();
    let mut store = wasmtime_selected::Store::new(&engine, ());
    let mut linker = wasmtime_selected::component::Linker::new(&engine);

    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap(
            "my-host-func",
            |_store, (a, b, c, d): (u32, u32, u32, u32)| Ok((host_impl(a, b, c, d),)),
        )
        .unwrap();

    let component = wasmtime_selected::component::Component::new(&engine, COMPONENT_WAT).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();
    let call_host = instance
        .get_typed_func::<(u32, u32, u32, u32), (u32,)>(&mut store, "call-host")
        .unwrap();

    group.bench_with_input(
        BenchmarkId::new("opa-embedded", case_arg_display(case)),
        case,
        |b, _| {
            b.iter(|| {
                let args = black_box((42u32, 7u32, 13u32, 99u32));
                black_box(call_host.call(&mut store, args).unwrap().0);
                black_box(call_host.post_return(&mut store).unwrap());
            })
        },
    );
}

#[cfg(feature = "branch-opa-test")]
fn opa_test_branch_case(group: &mut BenchmarkGroup<'_, WallTime>, case: &BranchCase) {
    let BranchPolicyConfig::OpaTest { opa_url } = case.config;

    let mut config = wasmtime_selected::Config::new();
    config.opa_url(opa_url);

    let engine = wasmtime_selected::Engine::new(&config).unwrap();
    let mut store = wasmtime_selected::Store::new(&engine, ());
    let mut linker = wasmtime_selected::component::Linker::new(&engine);

    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap(
            "my-host-func",
            |_store, (a, b, c, d): (u32, u32, u32, u32)| Ok((host_impl(a, b, c, d),)),
        )
        .unwrap();

    let component = wasmtime_selected::component::Component::new(&engine, COMPONENT_WAT).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();
    let call_host = instance
        .get_typed_func::<(u32, u32, u32, u32), (u32,)>(&mut store, "call-host")
        .unwrap();

    group.bench_with_input(
        BenchmarkId::new("opa-external", case_arg_display(case)),
        case,
        |b, _| {
            b.iter(|| {
                let args = black_box((42u32, 7u32, 13u32, 99u32));
                black_box(call_host.call(&mut store, args).unwrap().0);
                black_box(call_host.post_return(&mut store).unwrap());
            })
        },
    );
}

fn bench_wasmtime_host_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("guest_to_host_roundtrip");
    group.measurement_time(Duration::from_secs(15));

    #[cfg(feature = "branch-upstream")]
    for case in ACTIVE_CASES {
        v38_case(&mut group, case);
    }

    #[cfg(feature = "branch-blocking")]
    for case in ACTIVE_CASES {
        blocking_branch_case(&mut group, case);
    }

    #[cfg(feature = "branch-opa-local")]
    for case in ACTIVE_CASES {
        opa_local_branch_case(&mut group, case);
    }

    #[cfg(feature = "branch-opa-test")]
    for case in ACTIVE_CASES {
        opa_test_branch_case(&mut group, case);
    }

    group.finish();
}

criterion_group!(benches, bench_wasmtime_host_calls);
criterion_main!(benches);
