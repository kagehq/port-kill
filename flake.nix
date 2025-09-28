{
  description = "Port Kill - A CLI tool to help you find and free ports blocking your dev work";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit overlays system; };
        
        # Rust toolchain with specific version
        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu" "x86_64-apple-darwin" "aarch64-apple-darwin" ];
        };

        # Platform-specific dependencies
        linux-deps = with pkgs; [
          pkg-config
          gtk3
          libappindicator-gtk3
          atk
          gdk-pixbuf
          cairo
          pango
          gtk3.dev
          libxdo
        ];

        macos-deps = with pkgs; [
          # macOS dependencies are handled by the system
        ];

        windows-deps = with pkgs; [
          # Windows dependencies are minimal for Rust
        ];

        # Common build dependencies
        common-deps = with pkgs; [
          rust-toolchain
          cargo
          rustc
          pkg-config
        ];

        # Development dependencies
        dev-deps = with pkgs; [
          rust-toolchain
          cargo
          rustc
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

        # Packages for different platforms
        packages = {
          # Linux x86_64
          port-kill-linux = pkgs.rustPlatform.buildRustPackage {
            pname = "port-kill";
            version = "0.4.6";
            src = ./.;
            
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            buildInputs = linux-deps;
            
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            # Build both binaries
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
              platforms = platforms.linux;
            };
          };

          # macOS x86_64 (Intel)
          port-kill-macos-intel = pkgs.rustPlatform.buildRustPackage {
            pname = "port-kill";
            version = "0.4.6";
            src = ./.;
            
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # Cross-compile for Intel Mac
            target = "x86_64-apple-darwin";
            
            buildInputs = macos-deps;

            buildPhase = ''
              cargo build --release --target x86_64-apple-darwin --bin port-kill --bin port-kill-console
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/x86_64-apple-darwin/release/port-kill $out/bin/
              cp target/x86_64-apple-darwin/release/port-kill-console $out/bin/
            '';

            meta = with pkgs.lib; {
              description = "A CLI tool to help you find and free ports blocking your dev work";
              homepage = "https://github.com/kagehq/port-kill";
              license = licenses.mit;
              maintainers = [ ];
              platforms = [ "x86_64-darwin" ];
            };
          };

          # macOS aarch64 (Apple Silicon)
          port-kill-macos-arm64 = pkgs.rustPlatform.buildRustPackage {
            pname = "port-kill";
            version = "0.4.6";
            src = ./.;
            
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # Cross-compile for Apple Silicon Mac
            target = "aarch64-apple-darwin";
            
            buildInputs = macos-deps;

            buildPhase = ''
              cargo build --release --target aarch64-apple-darwin --bin port-kill --bin port-kill-console
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/aarch64-apple-darwin/release/port-kill $out/bin/
              cp target/aarch64-apple-darwin/release/port-kill-console $out/bin/
            '';

            meta = with pkgs.lib; {
              description = "A CLI tool to help you find and free ports blocking your dev work";
              homepage = "https://github.com/kagehq/port-kill";
              license = licenses.mit;
              maintainers = [ ];
              platforms = [ "aarch64-darwin" ];
            };
          };

          # Windows x86_64
          port-kill-windows = pkgs.rustPlatform.buildRustPackage {
            pname = "port-kill";
            version = "0.4.6";
            src = ./.;
            
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # Cross-compile for Windows
            target = "x86_64-pc-windows-gnu";
            
            buildInputs = windows-deps;

            buildPhase = ''
              cargo build --release --target x86_64-pc-windows-gnu --bin port-kill --bin port-kill-console
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/x86_64-pc-windows-gnu/release/port-kill.exe $out/bin/
              cp target/x86_64-pc-windows-gnu/release/port-kill-console.exe $out/bin/
            '';

            meta = with pkgs.lib; {
              description = "A CLI tool to help you find and free ports blocking your dev work";
              homepage = "https://github.com/kagehq/port-kill";
              license = licenses.mit;
              maintainers = [ ];
              platforms = [ "x86_64-windows" ];
            };
          };
        };

        # Default package (current platform)
        defaultPackage = if pkgs.stdenv.isLinux then self.packages.${system}.port-kill-linux
                        else if pkgs.stdenv.isDarwin then 
                          if pkgs.stdenv.isAarch64 then self.packages.${system}.port-kill-macos-arm64
                          else self.packages.${system}.port-kill-macos-intel
                        else if pkgs.stdenv.isWindows then self.packages.${system}.port-kill-windows
                        else self.packages.${system}.port-kill-linux;

        # Checks for CI
        checks = {
          build-linux = self.packages.${system}.port-kill-linux;
          build-macos-intel = self.packages.${system}.port-kill-macos-intel;
          build-macos-arm64 = self.packages.${system}.port-kill-macos-arm64;
          build-windows = self.packages.${system}.port-kill-windows;
        };
      });
}
