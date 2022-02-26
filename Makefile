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

francium = francium/target/$(target)-kernel/release/francium_$(board)
sm = modules/sm/target/$(target)-user/release/sm
fs = modules/fs/target/$(target)-user/release/fs
test = modules/test/target/$(target)-user/release/test

ifeq ($(arch), aarch64)
target=aarch64-unknown-francium
gdb=aarch64-none-elf-gdb
qemu_args=-M virt -cpu cortex-a53 -kernel $(francium) -serial stdio -m 512
else ifeq ($(arch), x86_64)
target=x86_64-unknown-francium
qemu_args=-M pc-q35-6.1 -kernel $(francium) -serial stdio -m 512
gdb=gdb
endif

.PHONY: qemu gdb $(francium) $(fs) $(test) clean

all: $(francium)
$(francium): $(fs) $(sm) $(test)
	echo $(target); \
	cd francium && cargo build --release --features=platform_$(board) --target=../$(target)-kernel.json

# todo rpi4 only
kernel8.bin: $(francium)
	aarch64-none-elf-objcopy -O binary $(francium) kernel8.bin

$(fs):
	cd modules/fs && cargo build --release --target=../../$(target)-user.json

$(sm):
	cd modules/sm && cargo build --release --target=../../$(target)-user.json

$(test):
	cd modules/test && cargo build --release --target=../../$(target)-user.json

qemu: $(francium)
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