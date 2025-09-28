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
      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            pkg-config
          ];
          
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
          '';

          RUST_BACKTRACE = "1";
          CARGO_TARGET_DIR = "./target";
        };

        # Simple build
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "port-kill";
          version = "0.3.11";
          src = ./.;
          
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
          ];

          buildPhase = ''
            cargo build --release --bin port-kill --bin port-kill-console
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/port-kill $out/bin/
            cp target/release/port-kill-console $out/bin/
          '';

          meta = with pkgs.lib; {
            description = "A CLI tool to help you find and free ports blocking your dev work";
            homepage = "https://github.com/kagehq/port-kill";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

        checks.default = self.packages.${system}.default;
      });
}