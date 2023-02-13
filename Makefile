board ?= virt

CARGO ?= cargo +francium

ifeq ($(board), virt)
arch=aarch64
target=aarch64-unknown-francium
kernel_target=aarch64-unknown-none-softfloat
else ifeq ($(board), raspi4)
target=aarch64-unknown-francium
kernel_target=aarch64-unknown-none-softfloat
else ifeq ($(board), pc)
arch=x86_64
target=x86_64-unknown-francium
kernel_target=x86_64-unknown-none
else
$(error Bad board!)
endif

francium = target/$(kernel_target)/release/francium_$(board)
sm = target/$(target)/release/sm
fs = target/$(target)/release/fs
test = target/$(target)/release/test
pcie = target/$(target)/release/pcie
disp = target/$(target)/release/disp

bootimg_bios = target/release/bios.img
bootimg_uefi = target/release/uefi.img

ifeq ($(arch), aarch64)
target=aarch64-unknown-francium
gdb=RUST_GDB=aarch64-unknown-francium-gdb rust-gdb +francium
qemu_args=-M virt,gic-version=2 -cpu cortex-a53 -kernel $(francium) -serial stdio -m 2048 -device bochs-display

else ifeq ($(arch), x86_64)
target=x86_64-unknown-francium
qemu_args=-M q35 -bios /usr/share/edk2/x64/OVMF.fd -drive format=raw,file=$(bootimg_uefi),if=none,id=boot -device virtio-blk,serial=fee1dead,drive=boot -serial stdio -m 2048 -no-reboot -enable-kvm
#qemu_args=-M q35 -bios /usr/share/edk2/x64/OVMF.fd -drive format=raw,file=$(bootimg_uefi),if=none,id=nvme -device nvme,serial=fee1dead,drive=nvme -serial stdio -m 2048 -no-reboot -enable-kvm
#qemu_args=-M q35 -drive format=raw,file=$(bootimg_bios),if=none,id=nvme -device nvme,serial=fee1dead,drive=nvme -serial stdio -m 2048 -no-reboot -enable-kvm -d int
gdb=rust-gdb
endif

CARGO_FLAGS =

.PHONY: qemu gdb bochs $(francium) $(bootimg_bios) $(bootimg_uefi) $(fs) $(sm) $(test) $(pcie) clean clean-user clean-kernel

all: $(francium) $(if $(filter $(board),raspi4), kernel8.bin)
$(francium): $(fs) $(sm) $(test) $(pcie) $(disp)
	cargo build --package=francium_$(board) --release --target=$(kernel_target)

$(bootimg_bios) $(bootimg_uefi): $(francium)
	cargo run --package=francium_pc_bootimg --release

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

$(pcie):
	$(CARGO) build --package=pcie --release --target=$(target)

$(disp):
	$(CARGO) build --package=disp --release --target=$(target)

qemu: $(francium) $(if $(filter $(board),pc), $(bootimg_uefi))
	qemu-system-$(arch) $(qemu_args) -s

ifeq ($(board), pc)
bochs: $(bootimg_bios)
	cp $(bootimg_bios) $(bootimg_bios)_bochs; \
	dd if=/dev/zero of=$(bootimg_bios)_bochs conv=notrunc bs=1 seek=67092479 count=1; \
	rm $(bootimg_bios)_bochs.lock; \
	bochs
endif

qemu-gdb: $(francium) $(if $(filter $(board),pc), $(bootimg_uefi))
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
	cd francium && $(CARGO) clean -p francium_kernel && cd ..

clean-user:
	$(CARGO) clean -p process --target=$(target) && $(CARGO) clean -p fs --release --target=$(target) && $(CARGO) clean -p sm --release --target=$(target) && $(CARGO) clean -p test --release --target=$(target) && $(CARGO) clean -p disp --release --target=$(target)
