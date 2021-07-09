
NAME=remilia

BOOT_IMAGE=bootimage-$(NAME).bin
KERNEL_IMAGE=$(NAME)

TARGET=x86_64-remilia
TARGET_JSON=.cargo/x86_64-remilia.json
TARGET_DIR=target/$(TARGET)/debug

QEMU_MEM=512
QEMU=qemu-system-x86_64
QEMU_ARGS=-drive format=raw,file=$(TARGET_DIR)/$(IMAGE_PATH) -serial stdio -m $(QEMU_MEM)
QEMU_TEST_ARGS=

BUILD_STD=-Zbuild-std=core,compiler_builtins,alloc -Zbuild-std-features=compiler-builtins-mem
ZFLAGS=$(BUILD_STD)
RSFLAGS= --target $(TARGET_JSON) $(ZFLAGS)

.PHONY: build

run: build
	CARGO_MANIFEST_DIR=$(PWD) bootimage runner "$(TARGET_DIR)/$(KERNEL_IMAGE)"

build:
	cargo build $(RSFLAGS)

check:
	cargo check $(RSFLAGS)

test:
	cargo test $(RSFLAGS)

test-patchouli:
	cd patchouli; cargo test
