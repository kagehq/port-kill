{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt
    
    # Build tools
    pkg-config
    
    # Platform-specific dependencies
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
    # Linux GUI dependencies
    gtk3
    libappindicator-gtk3
    atk
    gdk-pixbuf
    cairo
    pango
    libxdo
  ];

  # Environment variables
  RUST_BACKTRACE = "1";
  CARGO_TARGET_DIR = "./target";

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
}
