{
  description = "Port Kill - A CLI tool to help you find and free ports blocking your dev work";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit overlays system; };
        
        # Platform-specific dependencies
        linux-deps = with pkgs; [
          pkg-config
          gtk3
          libappindicator-gtk3
          atk
          gdk-pixbuf
          cairo
          pango
          libxdo
        ];

        macos-deps = with pkgs; [
          # macOS dependencies are handled by the system
        ];

        windows-deps = with pkgs; [
          # Windows dependencies are minimal for Rust
        ];

        # Development dependencies
        dev-deps = with pkgs; [
          rustc
          cargo
          rust-analyzer
          clippy
          rustfmt
          pkg-config
        ] ++ (if pkgs.stdenv.isLinux then linux-deps else [])
          ++ (if pkgs.stdenv.isDarwin then macos-deps else [])
          ++ (if pkgs.stdenv.isWindows then windows-deps else []);

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = dev-deps;
          
          shellHook = ''
            echo "🦀 Port Kill Development Environment"
            echo "=================================="
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build          - Build the project"
            echo "  cargo test           - Run tests"
            echo "  cargo clippy         - Run linter"
            echo "  cargo fmt            - Format code"
            echo ""
            echo "Platform-specific builds:"
            echo "  cargo build --target x86_64-unknown-linux-gnu"
            echo "  cargo build --target x86_64-pc-windows-gnu"
            echo "  cargo build --target x86_64-apple-darwin"
            echo "  cargo build --target aarch64-apple-darwin"
            echo ""
          '';

          # Set environment variables for reproducible builds
          RUST_BACKTRACE = "1";
          CARGO_TARGET_DIR = "./target";
        };

        # Single package for current platform
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "port-kill";
          version = "0.4.6";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = if pkgs.stdenv.isLinux then linux-deps
                      else if pkgs.stdenv.isDarwin then macos-deps
                      else if pkgs.stdenv.isWindows then windows-deps
                      else [];

          meta = with pkgs.lib; {
            description = "A CLI tool to help you find and free ports blocking your dev work";
            homepage = "https://github.com/kagehq/port-kill";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

        # Checks for CI
        checks.default = self.packages.${system}.default;
      });
}