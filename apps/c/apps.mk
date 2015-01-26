APPS := blink

HERE := $(dir $(call whereami))

app_src = $(call rwildcard,$(HERE)$(app)/,*.c)
APP_SRCS := $(foreach app,$(APPS),$(app_src))
APP_OBJS := $(APP_SRCS:.c=.o)

LIB_DIR := $(HERE)lib/
LIB_SRCS := $(call rwildcard,$(LIB_DIR),*.c)
LIB_OBJS := $(LIB_SRCS:.c=.o)

APP_INC := $(addsuffix /inc/,$(APPS))
LIB_INC := $(addsuffix inc/,$(LIB_DIR))
INCS := $(APP_INC) $(LIB_INC)
INC_FLAGS := $(addprefix -I,$(HERE)$(INCS))

APP_OBJECTS := $(APP_OBJS) $(LIB_OBJS)

.PHONY: all clean

$(APP_OBJS): $(APP_SRCS) $(LIB_OBJS)
$(LIB_OBJS): $(LIB_SRCS)

.c.o:
	@echo "+ $@"
	@$(CC) $(CFLAGS) $(INC_FLAGS) -c $< -o $@
