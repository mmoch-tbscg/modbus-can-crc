# CAN CRC Calculator

A high-performance CAN (Controller Area Network) CRC calculator implemented in Rust. This project provides both CLI and GUI interfaces for calculating CAN CRC checksums with optimized algorithms and parallel processing support.

## Features

- **CAN CRC-15 Implementation**: Implements the standard CAN CRC algorithm with polynomial 0x4599
- **Multiple Input Formats**: Supports both binary and hexadecimal input
- **High Performance**: Optimized with lookup tables and parallel processing for large iteration counts
- **Dual Interface**: Both command-line (CLI) and graphical user interface (GUI) versions
- **Benchmarking**: Built-in performance measurement with iteration support (1 to 1,000,000,000)
- **Input Validation**: Ensures input doesn't exceed 96 bits as per CAN specification

## Algorithm

The CAN CRC algorithm implemented follows the standard specification:

```
CRC_RG = 0;
REPEAT
    CRCNXT = NXTBIT EXOR CRC_RG(14)
    CRC_RG(14:1) = CRC_RG(13:0);
    CRC_RG(0) = 0;
    IF CRCNXT THEN
        CRC_RG(14:0) = CRC_RG(14:0) EXOR (4599hex);
    ENDIF
UNTIL (end of data)
```

## Building

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build Commands

```bash
# Clone the repository
cd can_crc_project

# Build all binaries
cargo build --release

# Build only CLI version
cargo build --release --bin cli

# Build only GUI version
cargo build --release --bin gui
```

## Usage

### Command Line Interface (CLI)

```bash
# Basic usage with hex input
cargo run --release --bin cli -- -d "AA BB CC" -i 1000000

# Using binary input
cargo run --release --bin cli -- -d "10101010 11110000" -f binary -i 1000

# Verbose output
cargo run --release --bin cli -- -d "01 04 00 00" -v -i 10000000

# Help
cargo run --release --bin cli -- --help
```

#### CLI Options:
- `-d, --data <DATA>`: Input data (binary or hex format)
- `-f, --format <FORMAT>`: Input format [default: hex] [possible values: binary, hex]
- `-i, --iterations <ITERATIONS>`: Number of iterations [default: 1]
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Print help information

### Graphical User Interface (GUI)

```bash
# Run the GUI version
cargo run --release --bin gui
```

The GUI provides:
- Radio buttons to switch between binary and hex input
- Text fields for data input
- Iteration count input with quick-select buttons
- Real-time CRC calculation
- Performance metrics display
- Example data buttons for quick testing

## Examples

### Example 1: Simple hex input
```bash
cargo run --release --bin cli -- -d "AA" -i 1
```
Output:
```
Results:
--------
CRC value (hex): 0x3A4D
CRC value (dec): 14925
CRC value (bin): 011101001001101

Performance:
------------
Total time: 0.001 ms
```

### Example 2: Binary input with performance testing
```bash
cargo run --release --bin cli -- -d "10101010" -f binary -i 1000000 -v
```

### Example 3: Maximum length input (96 bits = 12 bytes)
```bash
cargo run --release --bin cli -- -d "AA BB CC DD EE FF 00 11 22 33 44 55" -i 100000
```

## Performance

The implementation includes several optimizations:

1. **Lookup Table**: Pre-computed CRC values for all possible byte values
2. **Parallel Processing**: Automatic parallelization for iterations ≥ 100,000
3. **Bit-level Optimization**: Efficient bit manipulation operations

Typical performance on modern hardware:
- Single CRC calculation: < 1 microsecond
- 1 million iterations: ~10-50 ms (depending on data length and CPU)
- Parallel speedup: Near-linear with CPU core count for large iteration counts


## Project Structure

```
can_crc_project/
├── Cargo.toml          # Project configuration
├── README.md           # This file
└── src/
    ├── lib.rs          # Core CRC implementation
    ├── cli.rs          # Command-line interface
    └── gui.rs          # Graphical user interface
```

## Technical Details

- **CRC Polynomial**: 0x4599 (CAN standard)
- **CRC Width**: 15 bits
- **Maximum Input**: 96 bits (12 bytes)
- **Supported Formats**: Binary (0/1) and Hexadecimal
- **Parallelization Threshold**: 100,000 iterations