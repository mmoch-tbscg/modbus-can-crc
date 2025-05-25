#!/bin/bash

echo "Building CAN CRC Calculator..."
echo "=============================="

# Build in release mode
echo "Building CLI version..."
cargo build --release --bin cli

echo "Building GUI version..."
cargo build --release --bin gui

echo ""
echo "Build complete!"
echo ""
echo "Executables are located in:"
echo "  CLI: target/release/cli"
echo "  GUI: target/release/gui"
echo ""
echo "To run:"
echo "  ./target/release/cli --help"
echo "  ./target/release/gui" 