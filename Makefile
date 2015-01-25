RUSTC ?= rustc
RUSTC_FLAGS += -C opt-level=2 -Z no-landing-pads
RUSTC_FLAGS += --target config/thumbv7em-none-eabi
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check -Lbuild

VERSION_CMD = rustc --version | head -n 1 | sed 's/[^(]*(\([^ ]*\).*/\1/'
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
CORE_DIR=$(BUILD_DIR)/core-$(RUSTC_VERSION)
EXTERN_SRCS=extern

libs = $(addprefix $(BUILD_DIR)/lib,$(addsuffix .rlib,$(1)))
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) $(filter $(subst *,%,$2),$d))

all: $(SDB)

$(BUILD_DIR) $(CORE_DIR) $(EXTERN_SRCS):
	@mkdir -p $@

$(EXTERN_SRCS)/rustc-$(RUSTC_VERSION)-src.tar.gz: | $(EXTERN_SRCS)
	@echo "Fetching $(@F)"
	@wget -q -O $@ https://github.com/rust-lang/rust/archive/$(RUSTC_VERSION).tar.gz

$(EXTERN_SRCS)/rustc/src/libcore/lib.rs: $(EXTERN_SRCS)/rustc-$(RUSTC_VERSION)-src.tar.gz
	@echo "Untarring $(<F)"
	@mkdir -p $(EXTERN_SRCS)/rustc
	@tar -C $(EXTERN_SRCS)/rustc -zx --strip-components=1 -f $^
	@touch $@ # Touch so lib.rs appears newer than tarball

$(CORE_DIR)/libcore.rlib: $(EXTERN_SRCS)/rustc/src/libcore/lib.rs | $(CORE_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(CORE_DIR) $(EXTERN_SRCS)/rustc/src/libcore/lib.rs

$(BUILD_DIR)/libcore.rlib: $(CORE_DIR)/libcore.rlib | $(BUILD_DIR)
	@echo "Copying $< to $@"
	@cp $< $@

$(BUILD_DIR)/libplugins.rlib: $(call rwildcard,src/plugins/,*.rs) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) --out-dir $(BUILD_DIR) src/plugins/lib.rs

# Apps shouldn't depend on `platform`. Okay until drivers are more complete
$(BUILD_DIR)/libapps.rlib: $(call libs,core hil platform)
$(BUILD_DIR)/libplatform.rlib: $(call libs,core hil plugins)

$(BUILD_DIR)/lib%.rlib: $(call rwildcard,src/$*/,*.rs) $(call libs,core) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/$*/lib.rs

$(BUILD_DIR)/libdrivers.rlib: $(call rwildcard,src/drivers/,*.rs) $(call libs,core hil)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-blocks --out-dir $(BUILD_DIR) src/drivers/lib.rs

$(BUILD_DIR)/%.o: c/%.c | $(BUILD_DIR)
	@echo "Compiling $^"
	@$(CC) $(CFLAGS) -c -o $@ $^

$(BUILD_DIR)/main.o: $(RUST_SOURCES) $(call libs,core support platform drivers apps)
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
	rm -Rf $(BUILD_DIR) $(EXTERN_SRCS)
