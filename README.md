[这里有中文版](https://github.com/MimiRabbit87/tuff_virtual_machine/blob/master/README_zh-CN.md)
# Tuff Virtual Machine

Tuff Virtual Machine is a Rust implement of the CHIP-8 interpreter.

## What's Tuff?

Tuff is a collection of CHIP-8 toolchain projects, written in Rust, as my practice project. Tuff Virtual Machine is one of them.

## Features

- Full CHIP-8 instruction set
- Configurable CPU frequency (default 500Hz)
- Real-time debug display (registers, stack, PC, timers)
- Audio output
- Windowed rendering
- Supports `.ch8` ROM files

## Getting Started

### Download a Release

Check the [Releases](https://github.com/MimiRabbit87/tuff_virtual_machine/releases) page for pre-built executables (currently only Windows x64).  
If your platform is not provided, you can build from source.
Then type `path/to/tuff_virtual_machine -h` to check what you can do with it.

## Developer Instructions

### Build from Source

#### Prerequisites

- Rust (latest stable toolchain recommended)
- Cargo

#### Clone the Repository

```bash
git clone git@github.com:MimiRabbit87/tuff_virtual_machine.git
cd tuff_virtual_machine
```

#### Run

```bash
cd path/to/local_repository_cloned
cargo run -- -r path/to/your_rom_file.ch8
```

### Build

For debug build:

```bash
cd path/to/local_repository_cloned
cargo build
```

For optimized release build (much faster, recommended for regular use):

```bash
cd path/to/local_repository_cloned
cargo build --release
```

## Command Line Options

|Option|Description|
|----|----|
|`-r, --run <PATH>`|Path to the CHIP-8 ROM file to run|
|`-f, --frequency <HZ>`|Set CPU frequency (in instructions per second). Default is `500`. Use `0` for unlimited (unstable)|
|`-i, --with-debug-information`|Show debug information (registers, stack, timers) on the terminal|
|`--no-vertical-synchronization`|Disable `60Hz` frame sync (may cause rendering to be too fast)|
|`-h, --help`|Print help message (use `--help` for detailed help)|
|`-V, --version`|Print version information|

## Performance Tuning

 - For best performance, always build with `--release` when playing games.

 - Frequencies above `2KHz` may cause high CPU usage and system instability. Use with caution.

## Testing

Tuff Virtual Machine passes the following [Timendus test ROMs](https://github.com/Timendus/chip8-test-suite):

 - 1-chip8-logo.ch8

 - 2-ibm-logo.ch8

 - 3-corax+.ch8

 - 4-flags.ch8

 - 5-quirks.ch8

 - 6-keypad.ch8

 - 7-beep.ch8

## Contributing

Feel free to open issues or pull requests. Suggestions and improvements are welcome.

## License

This project is licensed under the MIT License – see the [LICENSE](https://github.com/MimiRabbit87/tuff_virtual_machine/blob/master/LICENSE) file for details.
