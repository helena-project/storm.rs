RUSTC ?= rustc
RUSTC_FLAGS += -g -A non_camel_case_types -A dead_code
LLC ?= llc
LLC_FLAGS = -mtriple arm-none-eabi -march=thumb -mcpu=cortex-m4 --asm-verbose=false

OBJCOPY ?= arm-none-eabi-objcopy
CC = arm-none-eabi-gcc
LD = arm-none-eabi-ld
CFLAGS += -g -mcpu=cortex-m4 -mthumb -g -nostdlib
LDFLAGS += -Tstormpayload.ld

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

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -c -o $@ $^

main.ll: $(RUST_SOURCES)
	$(RUSTC) $(RUSTC_FLAGS) --emit ir -o main.ll main.rs
	sed -i 's/"split-stack"/""/g' $@

%.o: %.ll
	$(LLC) $(LLC_FLAGS) -filetype=obj -o $@ $^

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

