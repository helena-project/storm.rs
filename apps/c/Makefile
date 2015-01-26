APPS := blink

app_src = $(call rwildcard,$(app)/,*.c)
APP_SRCS := $(foreach app,$(APPS),$(app_src))
APP_OBJS := $(APP_SRCS:.c=.o)

LIB_DIR := lib/
LIB_SRCS := $(call rwildcard,lib/,*.c)
LIB_OBJS := $(LIB_SRCS:.c=.o)

APP_INC := $(addsuffix /inc/,$(APPS))
LIB_INC := $(addsuffix inc/,$(LIB_DIR))
INCS := $(APP_INC) $(LIB_INC)
INC_FLAGS := $(addprefix -I,$(INCS))

APP_OBJECTS := $(APP_OBJS) $(LIB_OBJS)

.PHONY: all clean

all: $(APP_OBJECTS)
	@echo "Objects: $^"

$(APP_OBJS): $(LIB_OBJS)

.c.o:
	$(CC) $(CFLAGS) $(INC_FLAGS) -c $< -o $@

clean:
	rm -rf $(call rwildcard,,*.o)
