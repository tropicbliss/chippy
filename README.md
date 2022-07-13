# chippy

A simple and lightweight CHIP-8 emulator implementing the SUPER-CHIP specification with keyboard and sound support

## Compiling from source

If you are on another platform, compile the binary yourself to try it out:

```sh
git clone https://github.com/tropicbliss/chippy
cd chippy
cargo build --release
```

Compiling from source requires the latest stable version of Rust. Older Rust versions may be able to compile `chippy`, but they are not guaranteed to keep working.

The binary will be located in `target/release`.

## Usage

```
chippy 1.0.0
A simple and lightweight CHIP-8 emulator implementing the SUPER-CHIP specification with keyboard and sound support

USAGE:
    chippy.exe <ROM>

ARGS:
    <ROM>    Path to the ROM binary

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```