{
    description = "Rust 1.75 project using flake-utils (Darwin only)";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs, flake-utils, ... }:
        flake-utils.lib.eachSystem [ "x86_64-darwin" "aarch64-darwin" ] (system:
            let
                pkgs = import nixpkgs {
                    inherit system;
                    overlays = [
                        (final: prev: {
                            rust-bin = prev.rust-bin.stable."1.75.0";
                        })
                    ];
                };
                rustPlatform = pkgs.makeRustPlatform {
                    cargo = pkgs.rust-bin;
                    rustc = pkgs.rust-bin;
                };
            in {
                packages.default = rustPlatform.buildRustPackage {
                    pname = "your-project";
                    version = "0.1.0";
                    src = ./.;

                    cargoLock = {
                        lockFile = ./Cargo.lock;
                    };

                    meta = {
                        description = "Your Rust project";
                        platforms = pkgs.lib.platforms.darwin;
                    };
                };

                devShells.default = pkgs.mkShell {
                    buildInputs = [ pkgs.rust-bin ];
                    shellHook = ''
                        echo "Entering dev shell with Rust 1.75"
                    '';
                };
            });
}
