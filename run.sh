#!/usr/bin/env bash

set -e

# Create a temporary shell.nix if it doesn't exist
if [ ! -f shell.nix ]; then
cat > shell.nix <<'EOF'
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Core Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # System libraries
    glibc
    udev

    # Common native deps for Rust crates
    pkg-config
    openssl
    cmake
    gcc

    # Optional but commonly needed
    zlib
  ];

  # Fix for many crates needing pkg-config
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
}
EOF
fi

# Enter nix shell
echo "Running prototype"
nix-shell --run "cargo run"