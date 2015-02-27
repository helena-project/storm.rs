APP_CFLAGS := -mcpu=cortex-m4 -mthumb -mlittle-endian
APP_CFLAGS += -ffunction-sections -fdata-sections -fno-strict-aliasing
APP_CFLAGS += -fno-builtin -ffreestanding -nodefaultlibs -nostdlib -nostartfiles
APP_CFLAGS += -g -O3 -std=gnu99
PIC_CFLAGS := -fpic -msingle-pic-base -mpic-register=r6
PIC_CFLAGS += -mno-pic-data-is-text-relative
APP_LDFLAGS := -Tconfig/user_app.ld

HERE := $(dir $(call whereami))

app_src = $(call rwildcard,$(HERE)$(app)/,*.c)
APP_SRCS := $(foreach app,$(APPS),$(app_src))
APP_OBJS := $(APP_SRCS:.c=.bin.o)

LIB_DIR := $(HERE)lib/
LIB_SRCS := $(call rwildcard,$(HERE)/lib/,*.c)

APP_INC := $(addsuffix /inc/,$(APPS))
LIB_INC := $(addsuffix inc/,$(LIB_DIR))
INCS := $(APP_INC) $(LIB_INC)
INC_FLAGS := $(addprefix -I,$(HERE)$(INCS))

# Objects to be compiled with OS in main makefile
APP_OBJECTS += $(APP_OBJS)


$(APP_OBJS): $(APP_SRCS)

%.o: %.c
	@echo "+ building app: $(dir $<)"
	@$(CC) $(APP_CFLAGS) $(PIC_CFLAGS) $(INC_FLAGS) $(APP_LDFLAGS) $< $(LIB_SRCS) -o $@
# debug info
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(basename $@).lst

%.bin: %.o
	@$(OBJCOPY) -O binary --gap-fill 0xFF $< $@

%.bin.o: %.bin
	@$(LD) -r -b binary $< -o $@
	@$(OBJCOPY) --rename-section .data=.app $@
# debug info
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(basename $@).lst

