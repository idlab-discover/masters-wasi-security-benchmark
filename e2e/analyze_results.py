#!/usr/bin/env python3
import argparse
import glob
import json
import math
import os
import sys

def format_time(seconds):
    """Formats a time in seconds or milliseconds for readability."""
    if seconds is None:
        return "N/A"
    if seconds < 1.0:
        return f"{seconds * 1000:.2f} ms"
    else:
        return f"{seconds:.3f} s"

def format_mean_stddev(mean, stddev):
    """Formats mean and standard deviation consistently."""
    if mean is None or stddev is None:
        return "N/A"
    if mean < 1.0:
        return f"{mean * 1000:.2f} ms ± {stddev * 1000:.2f} ms"
    else:
        return f"{mean:.3f} s ± {stddev:.3f} s"

def main():
    # Set default directory to e2e/results/p1 relative to this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    default_dir = os.path.join(script_dir, "results", "p1")

    parser = argparse.ArgumentParser(description="Analyze hyperfine JSON benchmark results.")
    parser.add_argument(
        "target",
        nargs="?",
        default=default_dir,
        help=(
            f"A hyperfine JSON result file, or a directory containing JSON files "
            f"(default: {os.path.relpath(default_dir, script_dir) if script_dir else 'results/p1'})"
        ),
    )
    args = parser.parse_args()

    target = args.target

    if os.path.isfile(target):
        json_files = [target]
        label = target
    elif os.path.isdir(target):
        json_files = sorted(glob.glob(os.path.join(target, "*.json")))
        if not json_files:
            print(f"No JSON files found in '{target}'.", file=sys.stderr)
            sys.exit(0)
        label = target
    else:
        print(f"Error: '{target}' is neither a file nor a directory.", file=sys.stderr)
        sys.exit(1)

    print("=" * 100)
    print(f"Analyzing hyperfine results in: {label}")
    print("=" * 100)

    for json_file in json_files:
        file_name = os.path.basename(json_file)
        try:
            with open(json_file, "r") as f:
                data = json.load(f)
        except Exception as e:
            print(f"\n[Error reading {file_name}: {e}]", file=sys.stderr)
            continue

        results = data.get("results", [])
        if not results:
            print(f"\nFile: {file_name} (No results found)")
            continue

        # Sort results by mean time (fastest first)
        results.sort(key=lambda x: x.get("mean", float("inf")))

        print(f"\nBenchmark File: {file_name}")
        print("-" * 100)
        
        # Header: Command | Runs | Mean ± Stddev | Relative | Min ... Max
        headers = ["Command", "Runs", "Mean ± Stddev", "Relative", "Min ... Max"]
        
        fastest_mean = results[0].get("mean") if results else None
        fastest_stddev = results[0].get("stddev") if results else None
        
        # Gather all rows
        rows = []
        for r in results:
            cmd = r.get("command", "unknown")
            runs = str(len(r.get("times", [])))
            mean = r.get("mean")
            stddev = r.get("stddev")
            min_val = r.get("min")
            max_val = r.get("max")
            
            mean_std_str = format_mean_stddev(mean, stddev)
            min_max_str = f"{format_time(min_val)} ... {format_time(max_val)}"
            
            if fastest_mean and mean is not None and fastest_stddev is not None and stddev is not None:
                if mean == fastest_mean:
                    relative = "1.00"
                else:
                    ratio = mean / fastest_mean
                    std_ratio = ratio * math.sqrt((stddev / mean)**2 + (fastest_stddev / fastest_mean)**2)
                    relative = f"{ratio:.2f} ± {std_ratio:.2f}"
            else:
                relative = "N/A"
                
            rows.append((cmd, runs, mean_std_str, relative, min_max_str))

        # Dynamic spacing (with a minimum width for headers)
        col1_w = max(len(h[0]) for h in [headers] + rows)
        col2_w = max(len(h[1]) for h in [headers] + rows)
        col3_w = max(len(h[2]) for h in [headers] + rows)
        col4_w = max(len(h[3]) for h in [headers] + rows)
        col5_w = max(len(h[4]) for h in [headers] + rows)
        
        # Print header
        header_line = f"{headers[0].ljust(col1_w)} | {headers[1].ljust(col2_w)} | {headers[2].ljust(col3_w)} | {headers[3].ljust(col4_w)} | {headers[4].ljust(col5_w)}"
        print(header_line)
        print("-" * len(header_line))
        
        # Print rows
        for row in rows:
            print(f"{row[0].ljust(col1_w)} | {row[1].ljust(col2_w)} | {row[2].ljust(col3_w)} | {row[3].ljust(col4_w)} | {row[4].ljust(col5_w)}")
            
    print("\n" + "=" * 100)

if __name__ == "__main__":
    main()
