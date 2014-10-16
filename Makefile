RUSTC ?= rustc
LLC ?= llc
RUSTC_FLAGS = -A non_camel_case_types -A dead_code
LLC_FLAGS = -mtriple arm-none-eabi -march=thumb -mcpu=cortex-m4 --asm-verbose=false

OBJCOPY ?= arm-none-eabi-objcopy
CC = arm-none-eabi-gcc
LD = arm-none-eabi-ld
CFLAGS += -g -mcpu=cortex-m4 -mthumb -g -nostdlib
LDFLAGS += -Tstormpayload.ld

C_SOURCES=stormcrt1.c
C_OBJECTS=$(C_SOURCES:.c=.o)

RUST_SOURCES=main.rs
RUST_OBJECTS=$(RUST_SOURCES:%.rs=%.o)

EXECUTABLE=main.bin

all: main.bin

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $^

%.o: %.s
	$(CC) $(CFLAGS) -c -o $@ $^

%.ll: %.rs
	$(RUSTC) $(RUSTC_FLAGS) --emit ir -o $@ $^
	sed -i 's/"split-stack"/""/g' $@

%.s: %.ll
	$(LLC) $(LLC_FLAGS) $^ -o=$@

main.elf: $(RUST_OBJECTS) $(C_OBJECTS)
	$(LD) $(LDFLAGS) $^ -o main.elf

%.bin: %.elf
	$(OBJCOPY) -O binary $< $@
    
.PHONY: prog
program: main.bin
	JLinkExe prog.jlink || true

.PHONY: clean
clean:
	rm -f main.bin main.elf $(C_OBJECTS) $(RUST_OBJECTS) *.ll *.s

