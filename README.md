# Storm.rs

Rust code for the Storm

## Requirements

1. Rust
2. LLVM (a dependency of rust)
3. arm-non-eabi toolchain
4. JLinkExe (for programming the storm)

## Building

If all the tools are in your `$PATH`, you should be good to go. Otherwise set the env variables:

* `RUSTC` - `rustc` compiler
* `LLC` - llvm's `llc`
* `CC` - `arm-none-eabi-gcc`
* `LD` - `arm-none-eabi-LD`
* `OBJCOPY` - `arm-none-eabi-objcopy`

```bash
make
```

## Programming the storm

```bash
make program
```
