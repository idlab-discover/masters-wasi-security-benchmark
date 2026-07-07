#!/usr/bin/env bash
set -e

# Change directory to the script's directory so it runs correctly from anywhere
cd "$(dirname "$0")"

# Ensure the bin directory exists
mkdir -p bin

# Loop through all Rust source files in the src directory
for file in src/*.rs; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .rs)
        echo "Compiling $file -> bin/$name.wasm..."
        rustc --target wasm32-wasip2 -o bin/"$name".wasm src/"$name".rs
    fi
done
