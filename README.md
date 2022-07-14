# chippy

A cross-platform simple and lightweight CHIP-8 emulator implementing the SUPER-CHIP specification, with audio and input support. Also supports hires roms.

## Compiling from source

If you are on another platform, compile the binary yourself to try it out:

```sh
git clone https://github.com/tropicbliss/chippy
cd chippy
cargo build --release
```

Compiling from source requires the latest stable version of Rust. Older Rust versions may be able to compile `chippy`, but they are not guaranteed to keep working.

The binary will be located in `target/release`.

### Linux

```
# ubuntu system dependencies
apt install pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev

# fedora system dependencies
dnf install libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel

# arch linux system dependencies
pacman -S pkg-config libx11 libxi mesa-libgl alsa-lib
```

### Windows

On windows both MSVC and GNU target are supported, no additional dependencies required.

Also cross-compilation to windows from linux is supported:

```
rustup target add x86_64-pc-windows-gnu

cargo run --target x86_64-pc-windows-gnu
```
