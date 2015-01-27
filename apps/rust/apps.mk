HERE := $(dir $(call whereami))

APP_SRCS = $(call rwildcard,$(HERE),*.rs)
APP_OBJS := $(APP_SRCS:.c=.o)

APP_OBJECTS += $(BUILD_DIR)/rust-apps.o

$(BUILD_DIR)/rust-apps.o: $(APP_SRCS)
	@echo "Building Rust Apps..."
	@$(RUSTC) $(RUSTC_FLAGS) --emit obj -o $@ $(HERE)lib.rs
