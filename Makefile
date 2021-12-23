francium = francium/target/aarch64-unknown-francium-kernel/release/francium
cesium = cesium/target/aarch64-unknown-francium-user/release/cesium
hydrogen = hydrogen/target/aarch64-unknown-francium-user/release/hydrogen

QEMU_ARGS := -M virt -cpu cortex-a53 -kernel $(francium) -serial stdio -m 512

.PHONY: qemu gdb $(francium) $(cesium) $(hydrogen) clean

$(francium): $(cesium) $(hydrogen)
	cd francium && cargo build --release

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
