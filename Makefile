UF2CONV = ~/Projects/Embedded/bootloader/microsoft-uf2/utils/uf2conv.py
CARGO_OUT = target/thumbv6m-none-eabi/release/GO
BIN = builds/out.bin
UF2 = builds/out.uf2

$(BIN): $(CARGO_OUT)
	mkdir -p builds
	arm-none-eabi-objcopy -O binary $(CARGO_OUT) $(BIN)

.PHONY: $(CARGO_OUT) flash uf2_flash clean

$(CARGO_OUT):
	cargo build --release

uf2 : $(UF2)

$(UF2): $(BIN)
	$(UF2CONV) $(BIN) -b 0x2000 -f 0x68ed2b88 -o $(UF2)

PORT ?= /dev/ttyACM0

b_flash: $(BIN)
	sleep 2
	bossac --erase -w -v -b -R -U --offset=0x2000 -p $$(ls /dev/ttyACM* | head -1) $(BIN) || echo "flash failed - check port and bootloader"

flash: $(UF2)
	udisksctl mount -b $$(lsblk -o PATH,LABEL | awk '/GROSSBOOT/{print $$1}') 2>/dev/null || true
	$(UF2CONV) $(UF2) -f 0x68ed2b88 -D

clean:
	cargo clean
	rm -rf builds
