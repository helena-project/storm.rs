VERSION_CMD = rustc --version | head -n 1 | sed 's/[^(]*(\([^ ]*\).*/\1/'
RUSTC_VERSION=$(shell $(VERSION_CMD))

CORE_DIR=$(BUILD_DIR)/core-$(RUSTC_VERSION)
EXTERN_SRCS=extern

$(CORE_DIR) $(EXTERN_SRCS):
	@mkdir -p $@

$(EXTERN_SRCS)/rustc-$(RUSTC_VERSION)-src.tar.gz: | $(EXTERN_SRCS)
	@echo "Need libcore to compile Tock: fetching source $(@F)"
	wget -q -O $@ https://github.com/rust-lang/rust/archive/$(RUSTC_VERSION).tar.gz

$(EXTERN_SRCS)/rustc/src/lib%/lib.rs: $(EXTERN_SRCS)/rustc-$(RUSTC_VERSION)-src.tar.gz
	@echo "Untarring $(<F)"
	@rm -rf $(EXTERN_SRCS)/rustc
	@mkdir -p $(EXTERN_SRCS)/rustc
	@tar -C $(EXTERN_SRCS)/rustc -zx --strip-components=1 -f $^
	@touch $@ # Touch so lib.rs appears newer than tarball

$(CORE_DIR)/lib%.rlib: $(EXTERN_SRCS)/rustc/src/lib%/lib.rs | $(CORE_DIR)
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(CORE_DIR) $(EXTERN_SRCS)/rustc/src/lib$*/lib.rs

$(BUILD_DIR)/libcore.rlib: $(CORE_DIR)/libcore.rlib | $(BUILD_DIR)
	@echo "Copying $< to $@"
	@cp $< $@

# $(BUILD_DIR)/liballoc.rlib: $(CORE_DIR)/liballoc.rlib | $(BUILD_DIR)
# 	@echo "Copying $< to $@"
# 	@cp $< $@
