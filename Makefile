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
	@mkdir -p build/deps
	$(RUSTC) $(RUSTC_FLAGS) --out-dir build/deps $(RUST_LIBS_LOC)/lib$*/lib.rs

build/libhal.rlib: build/deps/libcore.rlib
build/libsupport.rlib: build/deps/libcore.rlib

build/lib%.rlib: src/%/*.rs
	@mkdir -p build
	$(RUSTC) $(RUSTC_FLAGS) --out-dir build src/$*/lib.rs

build/%.o: c/%.c
	$(CC) $(CFLAGS) -c -o $@ $^

build/main.o: $(RUST_SOURCES) build/deps/libcore.rlib build/libsupport.rlib build/libhal.rlib
	@mkdir -p build
	$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/main.rs

build/main.elf: build/main.o $(C_OBJECTS)
	$(CC) $(CFLAGS) $(LDFLAGS) $^ -lgcc -o $@

build/%.bin: build/%.elf
	$(OBJCOPY) -O binary $< $@

build/%.sdb: build/%.elf
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: prog
program: build/main.bin
	$(JLINK_EXE) prog.jlink || true

.PHONY: clean
clean:
	rm -Rf build

