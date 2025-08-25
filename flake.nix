{
    description = "Rust 1.75 project using flake-utils (Darwin only)";

    inputs = {
        flake-utils.url = "github:numtide/flake-utils";
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        fenix.url = "github:nix-community/fenix";
        # nixpkgs.url = "github:NixOS/nixpkgs/6f884c2#nodejs-slim";
        home-manager.url = "github:nix-community/home-manager/release-25.05";
        home-manager.inputs.nixpkgs.follows = "nixpkgs";
        xome.url = "github:jeff-hykin/xome";
        xome.inputs.nixpkgs.follows = "nixpkgs";
        xome.inputs.home-manager.follows = "home-manager";
    };

    outputs = { self, flake-utils, nixpkgs, fenix, home-manager, xome, ... }:
        flake-utils.lib.eachSystem [ "x86_64-darwin" "aarch64-darwin" ] (system:
            let
                pkgs = import nixpkgs { inherit system; };
                # rustToolchain = fenix.packages.${system}.stable.toolchain;
                rustToolchain = fenix.packages.${system}.toolchainOf {
                    channel = "1.75.0";
                    components = [ "rustc" "cargo" ];
                };
                rustPlatform = pkgs.makeRustPlatform {
                    cargo = rustToolchain;
                    rustc = rustToolchain;
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
