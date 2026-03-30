use std::sync::Arc;

use wasmtime::{Engine, Store, component::{Component, Linker, TypedFunc, WasmPolicy}};

pub const COMPONENT_WAT: &str = r#"
    (component
      (import "bench:host/api"
        (instance $host
          (export "my-host-func" (func $my-host-func (param "a" u32) (param "b" u32) (param "c" u32) (param "d" u32) (result u32)))
        )
      )

      (core module $m
        (import "" "my-host-func" (func $my-host-func (param i32 i32 i32 i32) (result i32)))
        (func (export "call-host") (param i32 i32 i32 i32) (result i32)
          local.get 0
          local.get 1
          local.get 2
          local.get 3
          call $my-host-func
        )
      )

      (core func $my-host-func (canon lower (func $host "my-host-func")))
      (core instance $i (instantiate $m
        (with "" (instance
          (export "my-host-func" (func $my-host-func))
        ))
      ))

      (func (export "call-host") (param "a" u32) (param "b" u32) (param "c" u32) (param "d" u32) (result u32)
        (canon lift (core func $i "call-host"))
      )
    )
"#;


pub fn instantiate_new_testcase(path: Option<&str>) -> (TypedFunc<(u32, u32, u32, u32), (u32,)>, Store<()>) {
    let policy: WasmPolicy;
    if path.is_none() {
        policy = WasmPolicy::new_no_file();
    } else {
        policy = serde_yaml::from_str::<WasmPolicy>(&std::fs::read_to_string(path.unwrap()).unwrap()).unwrap();
    }
    // 1. Setup the Wasmtime Engine and Store
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new_with_policy(&engine, Arc::new(policy));

    // 2. Define your idempotent host function
    // This simple function takes an i32, adds 1, and returns it.
    linker
        .instance("bench:host/api")
        .unwrap()
        .func_wrap(
            "my-host-func",
            |_store, (a, b, c, d): (u32, u32, u32, u32)| {
                Ok((
                    a.wrapping_mul(3)
                        .wrapping_add(b.wrapping_mul(5))
                        .wrapping_add(c.wrapping_mul(7))
                        .wrapping_add(d.wrapping_mul(11)),
                ))
            },
        )
        .unwrap();

    // 4. Compile and Instantiate
    let component = Component::new(&engine, COMPONENT_WAT).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();

    // 5. Call the component export that forwards to the lowered host function.
    let call_host = instance
        .get_typed_func::<(u32, u32, u32, u32), (u32,)>(&mut store, "call-host")
        .unwrap();

    (call_host, store)
}