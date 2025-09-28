# Nix Build Setup for Port Kill

This document describes how to use Nix for reproducible builds of Port Kill across different platforms.

## Benefits of Nix Builds

- **Reproducible**: Same inputs always produce the same outputs
- **Isolated**: Dependencies are managed in isolated environments
- **Cross-platform**: Build for multiple platforms from any system
- **Cached**: Build artifacts are cached for faster subsequent builds
- **Deterministic**: No "works on my machine" issues

## Quick Start

### Prerequisites

1. Install Nix: https://nixos.org/download.html
2. Enable flakes: `echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf`

### Development Environment

```bash
# Enter development shell
nix develop

# Or with legacy shell.nix
nix-shell
```

### Building

```bash
# Build for current platform
nix build

# Build for specific platform
nix build .#packages.x86_64-linux.port-kill-linux
nix build .#packages.x86_64-darwin.port-kill-macos-intel
nix build .#packages.aarch64-darwin.port-kill-macos-arm64
nix build .#packages.x86_64-windows.port-kill-windows

# Build all platforms
nix build .#packages.x86_64-linux.port-kill-linux .#packages.x86_64-darwin.port-kill-macos-intel .#packages.aarch64-darwin.port-kill-macos-arm64 .#packages.x86_64-windows.port-kill-windows
```

### Running

```bash
# After building, binaries are in ./result/bin/
./result/bin/port-kill --help
./result/bin/port-kill-console --help
```

## Available Packages

| Package | Platform | Architecture | Description |
|---------|----------|--------------|-------------|
| `port-kill-linux` | Linux | x86_64 | Linux binary |
| `port-kill-macos-intel` | macOS | x86_64 | Intel Mac binary |
| `port-kill-macos-arm64` | macOS | aarch64 | Apple Silicon binary |
| `port-kill-windows` | Windows | x86_64 | Windows binary |

## Development Workflow

### 1. Enter Development Shell

```bash
nix develop
```

This provides:
- Rust toolchain (rustc, cargo, rust-analyzer, clippy, rustfmt)
- Platform-specific dependencies (GTK, etc.)
- Environment variables for reproducible builds

### 2. Build and Test

```bash
# Standard cargo commands work
cargo build
cargo test
cargo clippy
cargo fmt

# Cross-compilation
cargo build --target x86_64-unknown-linux-gnu
cargo build --target x86_64-apple-darwin
cargo build --target aarch64-apple-darwin
cargo build --target x86_64-pc-windows-gnu
```

### 3. Build with Nix

```bash
# Build current platform
nix build

# Build specific platform
nix build .#packages.x86_64-linux.port-kill-linux
```

## CI/CD Integration

The repository includes a GitHub Actions workflow (`.github/workflows/nix-build.yml`) that:

- Builds all platforms using Nix
- Tests the resulting binaries
- Uploads artifacts for each platform
- Uses Nix caching for faster builds

## Advanced Usage

### Custom Rust Toolchain

To use a specific Rust version, modify `flake.nix`:

```nix
rust-toolchain = pkgs.rust-bin.stable."1.75.0".default.override {
  targets = [ "x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu" "x86_64-apple-darwin" "aarch64-apple-darwin" ];
};
```

### Adding Dependencies

Add new dependencies to the appropriate section in `flake.nix`:

```nix
linux-deps = with pkgs; [
  # existing deps...
  new-dependency
];
```

### Cross-compilation

Nix handles cross-compilation automatically. To build for a different platform:

```bash
# Build Linux binary on macOS
nix build .#packages.x86_64-linux.port-kill-linux

# Build Windows binary on Linux
nix build .#packages.x86_64-windows.port-kill-windows
```

## Troubleshooting

### Build Failures

1. **Missing dependencies**: Add them to the appropriate `*-deps` list in `flake.nix`
2. **Rust version issues**: Update the `rust-toolchain` in `flake.nix`
3. **Cross-compilation issues**: Ensure target is supported by Rust toolchain

### Performance

1. **Enable Nix caching**: Use `cachix` for faster builds
2. **Use `--show-trace`**: For detailed error information
3. **Build specific packages**: Only build what you need

### Common Commands

```bash
# Show available packages
nix flake show

# Show package details
nix show-derivation .#packages.x86_64-linux.port-kill-linux

# Build with verbose output
nix build .#packages.x86_64-linux.port-kill-linux --show-trace

# Clean build (no cache)
nix build .#packages.x86_64-linux.port-kill-linux --no-link

# Update dependencies
nix flake update
```

## Comparison with Traditional Builds

| Aspect | Traditional | Nix |
|--------|-------------|-----|
| Dependencies | Manual installation | Declarative, automatic |
| Reproducibility | Environment-dependent | Fully reproducible |
| Cross-compilation | Complex setup | Built-in support |
| Caching | Manual/CI-dependent | Automatic with cachix |
| Isolation | System-wide | Per-project |
| Rollback | Manual | Automatic with flakes |

## Migration from Traditional Builds

1. **Install Nix**: Follow the quick start guide
2. **Use development shell**: `nix develop` instead of manual dependency installation
3. **Build with Nix**: Use `nix build` instead of `cargo build`
4. **Update CI**: Use the provided Nix GitHub Actions workflow

## Contributing

When contributing to the Nix setup:

1. Test builds on multiple platforms
2. Update documentation for new dependencies
3. Ensure CI builds pass
4. Test both development and production builds

## Resources

- [Nix Manual](https://nixos.org/manual/nix/stable/)
- [Nix Flakes](https://nixos.wiki/wiki/Flakes)
- [Rust with Nix](https://nixos.wiki/wiki/Rust)
- [Cross-compilation with Nix](https://nixos.wiki/wiki/Cross_Compilation)
