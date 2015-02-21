RUSTC ?= rustc
RUSTC_FLAGS += -C opt-level=2 -Z no-landing-pads
RUSTC_FLAGS += --target config/thumbv7em-none-eabi
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4 -C relocation_model=static
RUSTC_FLAGS += -g -C no-stack-check -Lbuild

CC = arm-none-eabi-gcc
LD = arm-none-eabi-ld
OBJCOPY = arm-none-eabi-objcopy
OBJDUMP = arm-none-eabi-objdump

CFLAGS += -g -O3 -std=gnu99 -mcpu=cortex-m4 -mthumb -nostdlib
LDFLAGS += -Tconfig/stormpayload.ld
OBJDUMP_FLAGS := --disassemble-all --source --disassembler-options=force-thumb
OBJDUMP_FLAGS += -C -g --section-headers --disassemble-zeroes

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

whereami = $(CURDIR)/$(word $(words $(MAKEFILE_LIST)),$(MAKEFILE_LIST))
libs = $(addprefix $(BUILD_DIR)/lib,$(addsuffix .rlib,$(1)))
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
		  $(filter $(subst *,%,$2),$d))

all: build/main.elf

$(BUILD_DIR):
	@mkdir -p $@

# $(BUILD_DIR)/libcore.rlib
-include config/libcore.mk

# Load the list of apps to be included at build time
-include apps/applist.mk

# Compiles and adds to $(APP_OBJECTS)
-include apps/c/apps.mk

# Compiles and adds to $(APP_OBJECTS)
#-include apps/rust/apps.mk

$(BUILD_DIR)/libplugins.so: $(call rwildcard,src/plugins/,*.rs) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) --out-dir $(BUILD_DIR) src/plugins/lib.rs

$(BUILD_DIR)/libdrivers.rlib: $(call rwildcard,src/drivers/,*.rs) $(call libs,core hil)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -F unsafe-blocks --out-dir $(BUILD_DIR) src/drivers/lib.rs

$(BUILD_DIR)/libplatform.rlib: $(call libs,core hil) $(BUILD_DIR)/libplugins.so

.SECONDEXPANSION:
$(BUILD_DIR)/lib%.rlib: $$(call rwildcard,src/$$**/,*.rs) $(call libs,core) | $(BUILD_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/$*/lib.rs

$(BUILD_DIR)/%.o: c/%.c | $(BUILD_DIR)
	@echo "Compiling $^"
	@$(CC) $(CFLAGS) -c -o $@ $^

$(BUILD_DIR)/main.o: $(RUST_SOURCES) $(call libs,core support platform drivers)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o $@ src/main.rs

$(BUILD_DIR)/main.elf: $(BUILD_DIR)/main.o $(APP_OBJECTS) $(C_OBJECTS) $(ASM_OBJECTS)
	@echo "Linking $@"
	@$(CC) $(CFLAGS) $(LDFLAGS) $^ -o $@ -ffreestanding -lgcc -lc
# debug info
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(basename $@).lst

$(BUILD_DIR)/%.bin: $(BUILD_DIR)/%.elf
	@echo "$^ --> $@"
	@$(OBJCOPY) -O binary $< $@

$(BUILD_DIR)/%.sdb: $(BUILD_DIR)/%.elf
	@echo "Packing SDB..."
	@$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<

.PHONY: all program clean clean-all

# keep debugging files around
.PRECIOUS: %.lst

program: $(BUILD_DIR)/main.sdb
	sload flash $(BUILD_DIR)/main.sdb

clean:
	rm -Rf $(BUILD_DIR)/*.*
	@echo "rm -rf rwildcard: *.o"
	@rm -rf $(call rwildcard,,*.o)
	@rm -rf $(call rwildcard,,*.out)
	@rm -rf $(call rwildcard,,*.elf)
	@rm -rf $(call rwildcard,,*.bin)
	@rm -rf $(call rwildcard,,*.map)
	@rm -rf $(call rwildcard,,*.lst)

clean-all: clean
	rm -Rf $(BUILD_DIR) $(EXTERN_SRCS)
