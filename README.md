# Tock Embedded OS

Tock is an operating system designed for running multiple concurrent, mutually
distrustful applications on Cortex-M based embedded platforms like the
[Storm](http://storm.rocks). Tock's design centers around protection, both from
potentially malicious applications and from device drivers. Tock uses two
mechanisms to protect different components of the operating system. First, the
kernel and device drivers are written in Rust, a systems programming language
that provides compile-time memory safety, type safety and strict aliasing. Tock
uses Rust to protect the kernel (e.g. the scheduler and hardware abstraction
layer) from platform specific device drivers as well as isolate device drivers
from each other. Second, Tock uses memory protection units to isolate
applications from each other and the kernel.

## Requirements

1. [Rust](http://www.rust-lang.org/) 1.0-alpha.
3. [arm-non-eabi toolchain](https://launchpad.net/gcc-arm-embedded/).
4. stormloader (recommended) or JLinkExe for programming the storm.

### Installing Requirements

### Rust

The version of the Rust compiler you use should match the version checkout in
the `lib/rust` submodule. The easies way to make sure that is the case is to
compile it from source:

```bash
$ cd lib/rust
$ ./configure
$ make
$ sudo make install
```

However, compiling takes a long time. Instead we've mirrored the binary
distribution of the currently used version for various host operating systems:

  * [Linux 64-bit](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-x86_64-unknown-linux-gnu.tar.gz)
  * [Linux 32-bit](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-i686-unknown-linux-gnu.tar.gz)
  * [Mac 64-bit (.pkg)](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-x86_64-apple-darwin.pkg)
  * [Mac 32-bit (.pkg)](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-i686-apple-darwin.pkg)
  * [Mac 64-bit (.tar.gz)](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-x86_64-apple-darwin.tar.gz)
  * [Mac 32-bit (.tar.gz)](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-i686-apple-darwin.tar.gz)
  * [Windows 64-bit](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-x86_64-pc-windows-gnu.exe)
  * [Windows 32-bit](http://www.scs.stanford.edu/~alevy/rust/rust-1.0.0-alpha-i686-pc-windows-gnu.exe)

#### `arm-none-eabi` toolchain

On Mac OS X, you can get the arm-non-eabi toolchain via homebrew:

```bash
$ brew tap PX4/homebrew-px4
$ brew update
$ brew install gcc-arm-none-eabi-48
```

On Linux it is available through many distribution managers:

```bash
$ pacman -S arm-none-eabi-gcc
$ apt-get install gcc-arm-none-eabi
```

For Windows and other operating systems, download site is
[here](https://launchpad.net/gcc-arm-embedded/+download).

### Stormloader

You can obtain stormloader via pip:

```bash
sudo pip install stormloader
```

## Building

If all the tools are in your `$PATH`, you should be good to go. Otherwise set the env variables:

* `RUSTC` - `rustc` compiler
* `CC` - `arm-none-eabi-gcc`
* `OBJCOPY` - `arm-none-eabi-objcopy`

If this is a fresh checkout, you'll need to initialze the Rust submodule:

```bash
git submodule init
git submodule update
```

If you choose to compile Rust from source, you can build a copy from the
submodule:

```bash
$ pushd lib/rust
$ ./configure && make
```

Once built, you can either install the compiler for your machine, or just point
this build system to it:

```bash
$ popd
  # OS X
$ MY_PLAT=x86_64-apple-darwin
$ export RUSTC="DYLD_LIBRARY_PATH=lib/rust/$MY_PLAT/stage2/lib/ lib/rust/$MY_PLAT/stage2/bin/rustc"
  # Linux-ish
$ MY_PLAT=x86_64-unknown-linux-gnu
$ export RUSTC="LD_LIBRARY_PATH=lib/rust/$MY_PLAT/stage2/lib/ lib/rust/$MY_PLAT/stage2/bin/rustc"
```

You're now ready to build!

```bash
make
```

## Programming the storm

If you are using the stormloader, there is a `make` rule that compiles the
source and programs the storm:

```bash
make program
```

If you are using `JLinkExe`, use the script included in the source root
directory:

```bash
JLinkExe prog.jlink
```

