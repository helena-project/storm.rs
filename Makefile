RUSTC ?= rustc
RUSTC_FLAGS += --opt-level 2 -Z no-landing-pads
RUSTC_FLAGS += -C no-stack-check
RUSTC_FLAGS += -Ctarget-cpu=cortex-m4
RUSTC_FLAGS += -g -L. --target thumbv7em-none-eabi

RUST_LIB_CORE_LOC ?= $(HOME)/hack/rust/src/libcore/lib.rs

OBJCOPY ?= arm-none-eabi-objcopy
CC = arm-none-eabi-gcc
LD = arm-none-eabi-ld
CFLAGS += -g -mcpu=cortex-m4 -mthumb -g -nostdlib
LDFLAGS += -Tstormpayload.ld --gc-sections

C_SOURCES=stormcrt1.c
C_OBJECTS=$(C_SOURCES:.c=.o)

RUST_SOURCES=$(shell ls *.rs)
RUST_OBJECTS=$(RUST_SOURCES:%.rs=%.o)

SLOAD=sload
SDB=main.sdb

SDB_MAINTAINER=$(shell whoami)
SDB_VERSION=$(shell git show-ref -s HEAD)
SDB_NAME=storm.rs
SDB_DESCRIPTION="An OS for the storm"

JLINK_EXE=JLinkExe

all: $(SDB)

libcore.rlib:
	$(RUSTC) $(RUSTC_FLAGS) -o libcore.rlib $(RUST_LIB_CORE_LOC)

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -c -o $@ $^

main.o: $(RUST_SOURCES) libcore.rlib
	$(RUSTC) $(RUSTC_FLAGS) -C lto --emit obj -o main.o main.rs

main.elf: main.o $(C_OBJECTS)
	$(LD) $(LDFLAGS) $^ -o main.elf

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@

%.sdb: %.elf
	$(SLOAD) pack -m "$(SDB_MAINTAINER)" -v "$(SDB_VERSION)" -n "$(SDB_NAME)" -d $(SDB_DESCRIPTION) -o $@ $<
    
.PHONY: prog
program: main.bin
	$(JLINK_EXE) prog.jlink || true

.PHONY: clean
clean:
	rm -f main.bin main.elf $(C_OBJECTS) $(RUST_OBJECTS) *.ll *.s

