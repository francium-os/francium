board ?= virt

CARGO ?= cargo +francium

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
sm = target/$(target)-user/release/sm
fs = target/$(target)-user/release/fs
test = target/$(target)-user/release/test
bootimg = target/x86_64-unknown-francium-kernel/release/boot-bios-francium_pc.img

ifeq ($(arch), aarch64)
target=aarch64-unknown-francium
gdb=aarch64-unknown-francium-gdb
qemu_args=-M virt,gic-version=2 -cpu cortex-a53 -kernel $(francium) -serial stdio -m 2048
else ifeq ($(arch), x86_64)
target=x86_64-unknown-francium
qemu_args=-drive format=raw,file=$(bootimg) -serial stdio -m 2048 -no-reboot -enable-kvm
gdb=rust-gdb
endif

CARGO_FLAGS = -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem

.PHONY: qemu gdb bochs $(francium) $(bootimg) $(fs) $(sm) $(test) clean clean-user clean-kernel

all: $(francium) $(if $(filter $(board),raspi4), kernel8.bin)
$(francium): $(fs) $(sm) $(test)
	$(CARGO) build $(CARGO_FLAGS) --package=francium --release --features=platform_$(board) --target=targets/$(target)-kernel.json

$(bootimg): $(francium)
	$(CARGO) run --package=simple_boot target/x86_64-unknown-francium-kernel/release/francium_pc

ifeq ($(board), raspi4)
kernel8.bin: $(francium)
	aarch64-none-elf-objcopy -O binary $(francium) kernel8.bin
endif

$(fs):
	$(CARGO) build --package=fs --release --target=$(target)

$(sm):
	$(CARGO) build --package=sm --release --target=$(target)

$(test):
	$(CARGO) build --package=test --release --target=$(target)

qemu: $(francium) $(if $(filter $(board),pc), $(bootimg))
	qemu-system-$(arch) $(qemu_args) -s

ifeq ($(board), pc)
bochs: $(bootimg)
	cp $(bootimg) $(bootimg)_bochs; \
	dd if=/dev/zero of=$(bootimg)_bochs conv=notrunc bs=1 seek=67092479 count=1; \
	rm $(bootimg)_bochs.lock; \
	bochs
endif

qemu-gdb: $(francium) $(if $(filter $(board),pc), $(bootimg))
	qemu-system-$(arch) $(qemu_args) -s -S

gdb:
	$(gdb) $(francium) -ex 'target extended-remote localhost:1234'

openocd:
	sudo openocd -f interface/ftdi/minimodule.cfg -f pi4_openocd.cfg

openocd-dap:
	sudo openocd -f interface/cmsis-dap.cfg -f pi4_openocd.cfg

openocd-gdb:
	aarch64-none-elf-gdb $(francium) -ex 'target extended-remote localhost:3333'

clean: clean-user clean-kernel

clean-kernel:
	cd francium && $(CARGO) clean && cd ..

clean-user:
	$(CARGO) clean -p process --release --target=$(target) && $(CARGO) clean -p fs --release --target=$(target) && $(CARGO) clean -p sm --release --target=$(target) && $(CARGO) clean -p test --release --target=$(target)
