use atsamd_hal::pac::SCB;



const BOOTLOADER_MAGIC: u32 = 0xF016_69EF;
const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x2000_7FFC as *mut u32;

pub fn enter_bootloader() -> ! {

    unsafe {
        core::ptr::write_volatile(
            BOOTLOADER_MAGIC_ADDR,
            BOOTLOADER_MAGIC,
        );
    }
    cortex_m::asm::dsb();

    SCB::sys_reset();
}