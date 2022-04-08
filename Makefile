board ?= virt

ifeq ($(board), virt)
arch=aarch64
target=aarch64-unknown-francium
else ifeq ($(board), raspi4)
target=aarch64-unknown-francium
else ifeq ($(board), pc)
arch=x86_64
target=x86_64-unknown-francium
else
$(error Bad board!)
endif

francium = target/$(target)-kernel/release/francium_$(board)
sm = modules/sm/target/$(target)-user/release/sm
fs = modules/fs/target/$(target)-user/release/fs
test = modules/test/target/$(target)-user/release/test
bootimg = target/x86_64-unknown-francium-kernel/release/boot-bios-francium_pc.img

ifeq ($(arch), aarch64)
target=aarch64-unknown-francium
gdb=aarch64-none-elf-gdb
qemu_args=-M virt -cpu cortex-a53 -kernel $(francium) -serial stdio -m 2048
else ifeq ($(arch), x86_64)
target=x86_64-unknown-francium
qemu_args=-drive format=raw,file=$(bootimg) -serial stdio -m 2048 -no-reboot -d int
gdb=gdb
endif

CARGO_FLAGS = -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem

.PHONY: qemu gdb $(francium) $(bootimg) $(fs) $(test) clean 

all: $(francium) $(if $(filter $(board),raspi4), kernel8.bin)
$(francium): $(fs) $(sm) $(test)
	cargo build $(CARGO_FLAGS) --package=francium --release --features=platform_$(board) --target=targets/$(target)-kernel.json

$(bootimg): $(francium)
	cargo run --package=simple_boot target/x86_64-unknown-francium-kernel/release/francium_pc

ifeq ($(board), raspi4)
kernel8.bin: $(francium)
	aarch64-none-elf-objcopy -O binary $(francium) kernel8.bin
endif

$(fs):
	cargo build $(CARGO_FLAGS) --package=fs --release --target=targets/$(target)-user.json

$(sm):
	cargo build $(CARGO_FLAGS) --package=sm --release --target=targets/$(target)-user.json

$(test):
	cargo build $(CARGO_FLAGS) --package=test --release --target=targets/$(target)-user.json

qemu: $(francium) $(if $(filter $(board),pc), $(bootimg))
	qemu-system-$(arch) $(qemu_args) -s

qemu-gdb: $(francium)
	qemu-system-$(arch) $(qemu_args) -s -S

gdb:
	$(gdb) $(francium) -ex 'target extended-remote localhost:1234'

openocd:
	sudo openocd -f interface/ftdi/minimodule.cfg -f pi4_openocd.cfg

openocd-dap:
	sudo openocd -f interface/cmsis-dap.cfg -f pi4_openocd.cfg

openocd-gdb:
	aarch64-none-elf-gdb $(francium) -ex 'target extended-remote localhost:3333'

clean:
	cd francium && cargo clean && cd ..; \
	cd libprocess && cargo clean && cd ..
	cd modules/fs && cargo clean && cd ../..; \
	cd modules/sm && cargo clean && cd ../..; \
	cd modules/test && cargo clean && cd ../..
