francium = francium/target/aarch64-unknown-francium/debug/francium

QEMU_ARGS := -M virt -cpu cortex-a53 -kernel $(francium) -serial stdio -m 512

.PHONY: qemu gdb $(francium)
$(francium): 
	cd francium && cargo build

qemu: $(francium)
	qemu-system-aarch64 $(QEMU_ARGS) -s

qemu-gdb: $(francium)
	qemu-system-aarch64 $(QEMU_ARGS) -s -S

gdb:
	aarch64-none-elf-gdb $(francium) -ex 'target remote localhost:1234'
