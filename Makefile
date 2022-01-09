francium = francium/target/aarch64-unknown-francium-kernel/release/francium
cesium = cesium/target/aarch64-unknown-francium-user/release/cesium
hydrogen = hydrogen/target/aarch64-unknown-francium-user/release/hydrogen

QEMU_ARGS := -M virt -cpu cortex-a53 -kernel $(francium)_virt -serial stdio -m 512

.PHONY: qemu gdb $(francium)_virt $(francium)_raspi4 $(cesium) $(hydrogen) clean

all: $(francium)_virt $(francium)_raspi4

$(francium)_virt: $(cesium) $(hydrogen)
	cd francium && cargo build --release --features=platform_virt

$(francium)_raspi4: $(cesium) $(hydrogen)
	cd francium && cargo build --release --features=platform_raspi4

$(cesium):
	cd cesium && cargo build --release

$(hydrogen):
	cd hydrogen && cargo build --release

qemu: $(francium)
	qemu-system-aarch64 $(QEMU_ARGS) -s

qemu-gdb: $(francium)
	qemu-system-aarch64 $(QEMU_ARGS) -s -S

gdb:
	aarch64-none-elf-gdb $(francium) -ex 'target remote localhost:1234'

clean:
	cd francium && cargo clean && cd ..; \
	cd cesium && cargo clean && cd ..; \
	cd hydrogen && cargo clean && cd ..; \
	cd libprocess && cargo clean && cd ..
