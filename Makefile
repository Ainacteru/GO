BUILD_DIR := .build
GCC_DIR := gcc

# Build the project and copy outputs to .build
all: build


# Flash using OpenOCD from the build directory
flash:
	@echo -e "Flashing...! \n"	
	openocd -f interface/stlink.cfg -c "set CPUTAPID 0x0bc11477" -f target/at91samdXX.cfg -c "program $(BUILD_DIR)/AtmelStart.elf verify reset exit"

build:
	@echo "Building..."
	$(MAKE) -C $(GCC_DIR)
	mkdir -p $(BUILD_DIR)
	cp $(GCC_DIR)/AtmelStart.elf $(BUILD_DIR)/
	cp $(GCC_DIR)/AtmelStart.bin $(BUILD_DIR)/
	cp $(GCC_DIR)/AtmelStart.hex $(BUILD_DIR)/
	cp $(GCC_DIR)/AtmelStart.lss $(BUILD_DIR)/

upload: build flash


# Clean both GCC build and wrapper build folder
clean:
	$(MAKE) -C $(GCC_DIR) clean
	rm -rf $(BUILD_DIR)	