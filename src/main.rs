#![no_std]
#![no_main]

use core::sync::atomic::Ordering::Relaxed;

use atsamd_hal::delay::Delay;
use atsamd_hal::prelude::_atsamd_hal_embedded_hal_digital_v2_OutputPin;
use cortex_m::interrupt::free;
use defmt::info;

use defmt::warn;
use go::communcation::timestamp;
use go::communcation::usb::ON;
use go::communcation::usb::Usb;
use go::ehal::delay::DelayNs;
use go::ehal::digital::StatefulOutputPin;
use go::entry;
use go::pac::Interrupt;
use go::pac::NVIC;
use go::peripherals::i2c::I2c;
use go::sensors::bmp;
use go::sensors::bmp::Bmp;
use go::sensors::imu::Imu;
use go as bsp;
use bsp::hal;
use bsp::pac;

use hal::clock::GenericClockController;
use pac::{CorePeripherals, Peripherals};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);

    Usb::set_up(&mut clocks, &mut peripherals.pm, pins.usb_dm, pins.usb_dp, peripherals.usb);
    timestamp::set_up(&mut clocks, peripherals.tc3, &mut peripherals.pm);

    enable_interrupts();
    
    let mut led = pins.led.into_push_pull_output();
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let i2c = I2c::new_ref((pins.sda, pins.scl), peripherals.sercom3, &mut clocks, &mut peripherals.pm);

    let mut bmp = Bmp::new(&i2c);

    let mut imu = Imu::new(&i2c); 
    loop {
        // warn!("baro addr: 0x{:02x}, imu addr: 0x{:02x}", bmp.id(), imu.get_id());
        imu.init();

        if ON.load(Relaxed) {
            led.set_high();
        } else {
            led.set_low();
        }

        // free(|cs| {
        //     warn!("{:?}", *MESSAGE.borrow(cs).borrow());
        // });
        
        delay.delay_ms(500u32);
    }
}

fn enable_interrupts() {
    unsafe {
        NVIC::unmask(Interrupt::USB);
        NVIC::unmask(Interrupt::TC3);
    }
}