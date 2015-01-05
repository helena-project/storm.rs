# Storm.rs

Rust code for the Storm

## Requirements

1. [Rust](http://www.rust-lang.org/) -- 12/30/2014 nightly version available [here](http://www.scs.stanford.edu/~alevy/rust-2014-12-30.tar.gz)
2. [LLVM](http://llvm.org/) (a dependency of rust)
3. arm-non-eabi toolchain
4. JLinkExe (for programming the storm)
5. stormloader (for programming the storm)

### Sourcing Requirements

On Mac OS X, you can get the arm-non-eabi toolchain via homebrew:

```bash
brew tap PX4/homebrew-px4
brew update
brew install gcc-arm-none-eabi-48
```

You can obtain stormloader via pip:

```bash
sudo pip install stormloader
```

## Building

If all the tools are in your `$PATH`, you should be good to go. Otherwise set the env variables:

* `RUSTC` - `rustc` compiler
* `LLC` - llvm's `llc`
* `CC` - `arm-none-eabi-gcc`
* `LD` - `arm-none-eabi-LD`
* `OBJCOPY` - `arm-none-eabi-objcopy`

If this is a fresh checkout, you'll need to initialze the rust submodule:

```bash
git submodule init
git submodule update
```

For the moment, we're tracking the upstream rust compiler, which means you'll
have to build a copy for your local machine:

```bash
$ pushd lib/rust
$ ./configure && make
```

This won't be fast. Once built, you can either install the compiler for your
machine, or just point this build system to it:

```bash
$ popd
  # OS X
$ MY_PLAT=x86_64-apple-darwin
$ export RUSTC="DYLD_LIBRARY_PATH=lib/rust/$MY_PLAT/stage2/lib/ lib/rust/$MY_PLAT/stage2/bin/rustc"
  # Linux-ish
$ MY_PLAT=x86_64-unknown-linux-gnu
$ export RUSTC="LD_LIBRARY_PATH=lib/rust/$MY_PLAT/stage2/lib/ lib/rust/$MY_PLAT/stage2/bin/rustc"
```

Now you can build

```bash
make
```

## Programming the storm

```bash
make program
```
