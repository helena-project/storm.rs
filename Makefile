RUSTC ?= rustc
RUSTC_FLAGS += -C opt-level=2 -Z no-landing-pads
RUSTC_FLAGS += --target config/thumbv7em-none-eabi
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check -Lbuild

OBJCOPY ?= arm-none-eabi-objcopy
CC = arm-none-eabi-gcc
CFLAGS += -g -O3 -std=gnu99 -mcpu=cortex-m4 -mthumb -nostdlib
LDFLAGS += -Tconfig/stormpayload.ld

C_SOURCES=$(call rwildcard,src/support/,*.c)
C_OBJECTS=$(C_SOURCES:c/%.c=build/%.o)

ASM_SOURCES=$(call rwildcard,src/support/,*.S)
ASM_OBJECTS=$(ASM_SOURCES:S/%.c=build/%.o)

RUST_SOURCES=$(wildcard src/*.rs)
BUILD_DIR=build

SLOAD=sload
SDB=$(BUILD_DIR)/main.sdb
SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE=JLinkExe

UNAME = $(shell uname)
ifeq ($(UNAME),Linux)
DYLIB=so
else
DYLIB=dylib
endif

whereami = $(CURDIR)/$(word $(words $(MAKEFILE_LIST)),$(MAKEFILE_LIST))
libs = $(addprefix $(BUILD_DIR)/lib,$(addsuffix .rlib,$(1)))
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
		  $(filter $(subst *,%,$2),$d))

all: $(SDB)

$(BUILD_DIR):
	@mkdir -p $@

# $(BUILD_DIR)/libcore.rlib
-include config/libcore.mk

# Compiles and adds to $(APP_OBJECTS)
-include apps/c/apps.mk

# Compiles and adds to $(APP_OBJECTS)
-include apps/rust/apps.mk

platform_docs:
	rustdoc $(RUSTC_FLAGS) src/platform/lib.rs

$(BUILD_DIR)/libplugins.$(DYLIB): $(call rwildcard,src/plugins/,*.rs) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) --out-dir $(BUILD_DIR) src/plugins/lib.rs

$(BUILD_DIR)/libdrivers.rlib: $(call rwildcard,src/drivers/,*.rs) $(call libs,core hil)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-blocks --out-dir $(BUILD_DIR) src/drivers/lib.rs

$(BUILD_DIR)/libplatform.rlib: $(call libs,core hil) $(BUILD_DIR)/libplugins.$(DYLIB)

.SECONDEXPANSION:
$(BUILD_DIR)/lib%.rlib: $$(call rwildcard,src/$$**/,*.rs) $(call libs,core) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/$*/lib.rs

$(BUILD_DIR)/%.o: c/%.c | $(BUILD_DIR)
	@echo "Compiling $^"
	@$(CC) $(CFLAGS) -c -o $@ $^

$(BUILD_DIR)/main.o: $(RUST_SOURCES) $(call libs,core collections support platform drivers)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/main.rs

$(BUILD_DIR)/main.S: $(RUST_SOURCES) $(call libs,core support platform drivers)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit asm -o $@ src/main.rs

$(BUILD_DIR)/main.ir: $(RUST_SOURCES) $(call libs,core support platform drivers)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit llvm-ir -o $@ src/main.rs

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/main.o $(APP_OBJECTS) $(C_OBJECTS) $(ASM_OBJECTS)
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
	@echo "rm -rf rwildcard: *.o"
	@rm -rf $(call rwildcard,,*.o)

clean-all: clean
	rm -Rf $(BUILD_DIR) $(EXTERN_SRCS)
