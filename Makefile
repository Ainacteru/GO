UF2CONV = ~/Projects/Embedded/bootloader/microsoft-uf2/utils/uf2conv.py
DEVICE = GROSSBOOT
CARGO_OUT = target/thumbv6m-none-eabi/release/$(shell cat Cargo.toml | rg name | sed -E 's/.*\"(.*)\".*/\1/' )
# CARGO_OUT = target/thumbv6m-none-eabi/release/samd21-usb-defmt

OUTPUT = output
BIN = $(OUTPUT)/out.bin
UF2 = $(OUTPUT)/out.uf2

build: $(BIN) $(UF2)

$(BIN): $(CARGO_OUT)
	mkdir -p $(OUTPUT)
	arm-none-eabi-objcopy -O binary $(CARGO_OUT) $(BIN)

.PHONY: $(CARGO_OUT) flash uf2_flash clean

$(CARGO_OUT):
	cargo build --release

$(UF2): $(BIN)
	$(UF2CONV) $(BIN) -b 0x2000 -f 0x68ed2b88 -o $(UF2)

PORT ?= /dev/ttyACM0

b_flash: $(BIN)
	sleep 2
	bossac --erase -w -v -b -R -U --offset=0x2000 -p $$(ls /dev/ttyACM* | head -1) $(BIN) || echo "flash failed - check port and bootloader"

flash: $(UF2)
	udisksctl mount -b $$(lsblk -o PATH,LABEL | awk '/$(DEVICE)/{print $$1}') 2>/dev/null || true; \

# 	echo "Waiting for GROSSBOOT..."; \
# 	while ! mountpoint -q /run/media/Gary/GROSSBOOT; do \
# 		sleep 2; \
# 	done; \

# 	echo "Found GROSSBOOT!"; \
	
	$(UF2CONV) $(UF2) -f 0x68ed2b88 -D
	
clean:
	cargo clean
	rm -rf $(OUTPUT)
