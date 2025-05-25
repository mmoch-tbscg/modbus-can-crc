#!/bin/bash

echo "Building CAN CRC Calculator for Windows..."
echo "=========================================="

# Install Windows target if not already installed
rustup target add x86_64-pc-windows-gnu

# Build for Windows
echo "Building CLI version for Windows..."
cargo build --release --target x86_64-pc-windows-gnu --bin cli

echo "Building GUI version for Windows..."
cargo build --release --target x86_64-pc-windows-gnu --bin gui

echo ""
echo "Build complete!"
echo ""
echo "Windows executables are located in:"
echo "  CLI: target/x86_64-pc-windows-gnu/release/cli.exe"
echo "  GUI: target/x86_64-pc-windows-gnu/release/gui.exe" 