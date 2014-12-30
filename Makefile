RUSTC ?= rustc
RUSTC_FLAGS += --opt-level 2 -Z no-landing-pads
RUSTC_FLAGS += --target config/thumbv7em-none-eabi
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check -Lbuild/deps -Lbuild

RUST_LIBS_LOC ?= lib

OBJCOPY ?= arm-none-eabi-objcopy
CC = arm-none-eabi-gcc
LD = arm-none-eabi-ld
CFLAGS += -g -mcpu=cortex-m4 -mthumb -g -nostdlib
LDFLAGS += -Tconfig/stormpayload.ld

C_SOURCES=c/stormcrt1.c
C_OBJECTS=$(C_SOURCES:c/%.c=build/%.o)

ASM_SOURCES=c/ctx_switch.S
ASM_OBJECTS=$(ASM_SOURCES:S/%.c=build/%.o)

RUST_SOURCES=$(shell ls src/*.rs)

SLOAD=sload
SDB=build/main.sdb

SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE=JLinkExe

all: $(SDB)

build/deps/liballoc.rlib: build/deps/libcore.rlib

build/deps/lib%.rlib:
	@echo "Building $@"
	@mkdir -p build/deps
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir build/deps $(RUST_LIBS_LOC)/lib$*/lib.rs

build/libapps.rlib: build/deps/libcore.rlib build/libhil.rlib build/libhal.rlib
build/libhal.rlib: build/deps/libcore.rlib build/libhil.rlib
build/libsupport.rlib: build/deps/libcore.rlib

build/lib%.rlib: src/%/*.rs
	@echo "Building $@"
	@mkdir -p build
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir build src/$*/lib.rs

build/libdrivers.rlib: src/drivers/*.rs build/deps/libcore.rlib build/libhil.rlib
	@echo "Building $@"
	@mkdir -p build
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-blocks --out-dir build src/drivers/lib.rs

build/%.o: c/%.c
	@echo "Compiling $^"
	@$(CC) $(CFLAGS) -c -o $@ $^

build/main.o: $(RUST_SOURCES) build/deps/libcore.rlib build/libsupport.rlib build/libhal.rlib build/libdrivers.rlib build/libapps.rlib
	@echo "Building $@"
	@mkdir -p build
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/main.rs

build/main.elf: build/main.o $(C_OBJECTS) $(ASM_OBJECTS)
	@echo "Linking $@"
	@$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@ -ffreestanding -lgcc -lc

build/%.bin: build/%.elf
	@echo "$^ --> $@"
	@$(OBJCOPY) -O binary $< $@

build/%.sdb: build/%.elf
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: program
program: build/main.sdb
	sload flash build/main.sdb

.PHONY: clean
clean:
	rm -Rf build/*.*

.PHONY: clean-all
clean-all:
	rm -Rf build

