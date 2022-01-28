francium = francium/target/aarch64-unknown-francium-kernel/release/francium
cesium = cesium/target/aarch64-unknown-francium-user/release/cesium
hydrogen = hydrogen/target/aarch64-unknown-francium-user/release/hydrogen

QEMU_ARGS := -M virt -cpu cortex-a53 -kernel $(francium)_virt -serial stdio -m 512

.PHONY: qemu gdb $(francium)_virt $(francium)_raspi4 $(cesium) $(hydrogen) clean

all: $(francium)_virt kernel8.bin

$(francium)_virt: $(cesium) $(hydrogen)
	cd francium && cargo build --release --features=platform_virt

$(francium)_raspi4: $(cesium) $(hydrogen)
	cd francium && cargo build --release --features=platform_raspi4

kernel8.bin: $(francium)_raspi4
	aarch64-none-elf-objcopy -O binary $(francium)_raspi4 kernel8.bin

$(cesium):
	cd cesium && cargo build --release

$(hydrogen):
	cd hydrogen && cargo build --release

qemu: $(francium)_virt
	qemu-system-aarch64 $(QEMU_ARGS) -s

qemu-gdb: $(francium)_virt
	qemu-system-aarch64 $(QEMU_ARGS) -s -S

gdb:
	aarch64-none-elf-gdb $(francium)_virt -ex 'target extended-remote localhost:1234'

openocd:
	sudo openocd -f interface/ftdi/minimodule.cfg -f pi4_openocd.cfg

openocd-gdb:
	aarch64-none-elf-gdb $(francium)_raspi4 -ex 'target extended-remote localhost:3333'

clean:
	cd francium && cargo clean && cd ..; \
	cd cesium && cargo clean && cd ..; \
	cd hydrogen && cargo clean && cd ..; \
	cd libprocess && cargo clean && cd ..
