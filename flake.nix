{
  description = "Development environment for Python (CBOR/Matplotlib), Rust, and OPA";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      # Support standard Linux and macOS architectures
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      
      # Helper function to generate attributes for all systems
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          
          # Create a Python environment with the required packages
          pythonEnv = pkgs.python3.withPackages (ps: with ps; [
            matplotlib
            cbor2
          ]);

          # Define Rust toolchain with the wasm32-wasip2 target
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-wasip2" ];
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Python environment
              pythonEnv

              # Open Policy Agent
              open-policy-agent

              # Rust tooling (managed via rust-overlay to include wasm32-wasip2)
              rustToolchain

              # Benchmarking tool
              hyperfine
            ];

            shellHook = ''
              echo "🔨 Development Environment Loaded"
              echo "--------------------------------"
              echo "🐍 Python: $(python --version)"
              echo "🛡️  OPA:    $(opa version | head -n 1)"
              echo "🦀 Rust:   $(rustc --version)"
            '';
          };
        }
      );
    };
}