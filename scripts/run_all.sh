#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"${SCRIPT_DIR}/opa_test_instances.sh" stop

# warms up the core
taskset -c 4 cargo criterion --bench host_call --no-default-features --features branch-upstream

# thermal throttling should be mitigated
taskset -c 4 cargo criterion --bench host_call --no-default-features --features branch-upstream
taskset -c 4 cargo criterion --bench host_call --no-default-features --features branch-blocking
taskset -c 4 cargo criterion --bench host_call --no-default-features --features branch-opa-local
"${SCRIPT_DIR}/opa_test_instances.sh" start
taskset -c 4 cargo criterion --bench host_call --no-default-features --features branch-opa-test
"${SCRIPT_DIR}/opa_test_instances.sh" stop
