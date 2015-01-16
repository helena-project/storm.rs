RUSTC ?= rustc
RUSTC_FLAGS += -C opt-level=2 -Z no-landing-pads
RUSTC_FLAGS += --target config/thumbv7em-none-eabi
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check -Lbuild

VERSION_CMD = rustc --version | sed 's/[^(]*(\([^ ]*\).*/\1/'
RUSTC_VERSION=$(shell $(VERSION_CMD))

OBJCOPY ?= arm-none-eabi-objcopy
CC = arm-none-eabi-gcc
CFLAGS += -g -mcpu=cortex-m4 -mthumb -g -nostdlib
LDFLAGS += -Tconfig/stormpayload.ld

C_SOURCES=c/stormcrt1.c
C_OBJECTS=$(C_SOURCES:c/%.c=build/%.o)

ASM_SOURCES=c/ctx_switch.S
ASM_OBJECTS=$(ASM_SOURCES:S/%.c=build/%.o)

RUST_SOURCES=$(wildcard src/*.rs)

SLOAD=sload
SDB=build/main.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE=JLinkExe

BUILD_DIR=build
CORE_DIR=$(BUILD_DIR)/core

libs = $(addprefix $(BUILD_DIR)/lib,$(addsuffix .rlib,$(1)))

all: $(SDB)

$(BUILD_DIR) $(CORE_DIR):
	@mkdir -p $@

$(CORE_DIR)/rustc-$(RUSTC_VERSION)-src.tar.gz: | $(CORE_DIR)
	@echo "Fetching $(@F)"
	@mkdir -p $(CORE_DIR)/rustc
	@wget -q -O $@ https://github.com/rust-lang/rust/archive/$(RUSTC_VERSION).tar.gz

# Overrides `opt-level` because for some reason `opt-level=2` includes an
# `__aeabi_memset` (an GCC intrinsic available during linking) that doesn't get
# resolved when compiling main.o, even though it seems to be available for some
# other crates. This works for now, but should be fixed ASAP, this is obviously
# a stupid bug.
$(CORE_DIR)/libcore.rlib: $(CORE_DIR)/rustc-$(RUSTC_VERSION)-src.tar.gz
	@echo "Untarring $(<F)"
	@tar -C $(CORE_DIR)/rustc -zx --strip-components=1 -f $^
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C opt-level=0 --out-dir $(CORE_DIR) $(CORE_DIR)/rustc/src/libcore/lib.rs

$(BUILD_DIR)/libcore.rlib: $(CORE_DIR)/libcore.rlib | $(BUILD_DIR)
	@echo "Copying $< to $@"
	@cp $< $@

$(BUILD_DIR)/libapps.rlib: $(call libs,core hil hal)
$(BUILD_DIR)/libhal.rlib: $(call libs,core hil)

$(BUILD_DIR)/lib%.rlib: src/%/*.rs $(call libs,core) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/$*/lib.rs

$(BUILD_DIR)/libdrivers.rlib: src/drivers/*.rs $(call libs,core hil)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-blocks --out-dir $(BUILD_DIR) src/drivers/lib.rs

$(BUILD_DIR)/%.o: c/%.c | $(BUILD_DIR)
	@echo "Compiling $^"
	@$(CC) $(CFLAGS) -c -o $@ $^

$(BUILD_DIR)/main.o: $(RUST_SOURCES) $(call libs,core support hal drivers apps)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/main.rs

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/main.o $(C_OBJECTS) $(ASM_OBJECTS)
	@echo "Linking $@"
	@$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@ -ffreestanding -lgcc -lc

$(BUILD_DIR)/%.bin: $(BUILD_DIR)/%.elf
	@echo "$^ --> $@"
	@$(OBJCOPY) -O binary $< $@

$(BUILD_DIR)/%.sdb: $(BUILD_DIR)/%.elf
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: all program clean clean-all

program: $(BUILD_DIR)/main.sdb
	sload flash $(BUILD_DIR)/main.sdb

clean:
	rm -Rf $(BUILD_DIR)/*.*

clean-all:
	rm -Rf $(BUILD_DIR)
