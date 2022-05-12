TARGET := riscv64gc-unknown-linux-gnu
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
BUILD_DIR := ../riscv_rootfs/home/
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
PY := python3

APPS := $(wildcard $(APP_DIR)/*.rs)
APPS_C := $(wildcard $(APP_DIR)/*.c)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))

elf: $(APPS)
	@cargo build --release

binary: elf
	@$(foreach elf, $(ELFS), \
		$(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf)); \
		cp $(elf) $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.elf, $(elf));)

pre:
	@mkdir -p $(BUILD_DIR)/bin/
	
build: clean pre binary build_c
	@$(foreach t, $(ELFS), cp $(t) $(BUILD_DIR)/bin/;)

build_c:
	@$(foreach t, $(APPS_C), riscv64-unknown-linux-gnu-gcc $(t) -static-libgcc -Wl,-Bstatic -o $(BUILD_DIR)/bin/$(subst .c, ,$(notdir $(t)));)


clean:
	@cargo clean
	@rm -rf $(BUILD_DIR)

.PHONY: elf binary build clean
