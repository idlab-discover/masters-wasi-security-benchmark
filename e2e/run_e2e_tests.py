import argparse
import os
import subprocess
import sys

runtimes = {
    "v38": "test_runtimes/v38_0_3/wasmtime",
    "p1": "test_runtimes/policy1/wasmtime",
    "p2": "test_runtimes/policy2/wasmtime",
    "opa_e": "test_runtimes/opa_external/wasmtime",
    "opa_l": "test_runtimes/opa_local/wasmtime",
}

# "bin" is the shared wasm binary to run
# other entries are the specific arguments to pass to the runtime
testcases = {
    "startup": {
        "bin": "guests/bin/noop.wasm",
        "v38": "",
        "p1": "",
        "p2": "",
        "opa_e": "",
        "opa_l": "",
    },
    "fs": {"bin": "guests/bin/fs_benchmark.wasm"},  # TODO: fill in arguments
    "net": {"bin": "guests/bin/net_benchmark.wasm"},  # TODO: fill in arguments
    # TODO : create more testcases
}

def main():
    # Change working directory to the script's directory so relative paths work
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)

    parser = argparse.ArgumentParser(description="Run end-to-end benchmark tests using hyperfine.")
    parser.add_argument("--testcase", required=True, choices=list(testcases.keys()), help="Which testcase to run")
    parser.add_argument("--warmup", help="Warmup runs for hyperfine (e.g., --warmup 3)")
    parser.add_argument("--runs", help="Number of runs for hyperfine (e.g., --runs 10)")
    parser.add_argument("runtimes", nargs="*", help="Runtimes to run. If none are specified, runs all available runtimes.")

    args = parser.parse_args()

    # Determine which runtimes to run
    selected_runtimes = args.runtimes
    if not selected_runtimes:
        selected_runtimes = list(runtimes.keys())
    else:
        # Validate selected runtimes
        invalid_runtimes = [r for r in selected_runtimes if r not in runtimes]
        if invalid_runtimes:
            print(f"Error: Unknown runtime(s): {', '.join(invalid_runtimes)}", file=sys.stderr)
            print(f"Available runtimes: {', '.join(runtimes.keys())}", file=sys.stderr)
            sys.exit(1)

    # Get the testcase details
    testcase_name = args.testcase
    testcase_data = testcases[testcase_name]
    bin_path = testcase_data.get("bin")
    if not bin_path:
        print(f"Error: Testcase '{testcase_name}' is missing 'bin' entry.", file=sys.stderr)
        sys.exit(1)

    # Build the hyperfine command
    hyperfine_cmd = ["hyperfine"]
    if args.warmup is not None:
        hyperfine_cmd.extend(["--warmup", str(args.warmup)])
    if args.runs is not None:
        hyperfine_cmd.extend(["--runs", str(args.runs)])

    # Construct the commands to compare
    cases_to_run = []
    for r in selected_runtimes:
        executable = runtimes[r]
        specific_args = testcase_data.get(r, "")
        
        # Build the command string for the runtime
        parts = [executable]
        if specific_args:
            parts.append(specific_args)
        parts.append(bin_path)
        
        cmd_str = " ".join(parts)
        cases_to_run.append(cmd_str)

    hyperfine_cmd.extend(cases_to_run)

    print(f"Running hyperfine command: {' '.join(hyperfine_cmd)}")
    subprocess.run(hyperfine_cmd)

if __name__ == "__main__":
    main()

