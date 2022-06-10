TARGET := riscv64gc-unknown-none-elf
MODE := release
ENV := host
APP_DIR := $(ENV)/src/bin
TARGET_DIR := $(ENV)/target/$(TARGET)/$(MODE)
BUILD_DIR := ../riscv_rootfs/home/bin
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
PY := python3

TEST ?= 0
ifeq ($(TEST), 0)
	APPS :=  $(filter-out $(wildcard $(APP_DIR)/test*.rs), $(wildcard $(APP_DIR)/*.rs))
else
	APPS :=  $(wildcard $(APP_DIR)/test$(TEST)*.rs)
endif
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))

elf: $(APPS)
	@cd $(ENV) && cargo build --release

binary: elf
	@$(foreach elf, $(ELFS), \
		$(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf)); \
		cp $(elf) $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.elf, $(elf));)

pre:
	@mkdir -p $(BUILD_DIR)/
	
build: clean pre binary
	@$(foreach t, $(ELFS), cp $(t).elf $(BUILD_DIR);)

clean:
	@cargo clean

.PHONY: elf binary build clean