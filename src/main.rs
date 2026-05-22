#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embassy_executor::Spawner;
use go::Led;
use go::RgbRed;
use go::actuators::servo::Servo;
use go::entry;
use go::indicators::Leds::rgb_leds;
use go::indicators::Leds::rgb_leds::RgbLed;
use atsamd_hal::delay::Delay;
use atsamd_hal::pwm::Pwm0;

use go::sensors::bmp::Bmp;
use go as bsp;
use atsamd_hal::pwm::Pwm2;
use bsp::hal;
use bsp::pac;

use hal::clock::GenericClockController;
use hal::prelude::*;
use pac::{CorePeripherals, Peripherals};

use go::{uprintln, uprint};
use go::communcation::usb;
use go::peripherals::i2c::I2c;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);

    usb::set_up(
        peripherals.usb,
        &mut clocks,
        &mut peripherals.pm,
        pins.usb_dm,
        pins.usb_dp,
        &mut core.NVIC,
    );

    let glck0 = clocks.gclk0();

    
    let mut delay = Delay::new(core.SYST, &mut clocks);


    delay.delay_ms(2000u32);
    usb::set_input_ready();
    uprint!("starting");

    
    let i2c = I2c::new((pins.sda.into(), pins.scl.into()), peripherals.sercom3, &mut clocks, &mut peripherals.pm);
    let mut baro = Bmp::new(i2c);

    loop {
        uprintln!("temperature: {}", baro.read_temperature());
    }
}

#[panic_handler]
fn panic (_info: &PanicInfo) -> ! {
    let peripherals = unsafe { Peripherals::steal() };
    let pins = bsp::Pins::new(peripherals.port);
    let mut led: Led = pins.led.into();
    let mut red: RgbRed = pins.rgb_red.into();
    unsafe { cortex_m::interrupt::enable() };

    

    red.set_high();

    loop {
        led.toggle();
        uprintln!("PANIC! {}", _info);

        cortex_m::asm::delay(6_500_000); // ~1s at 8MHz
    }
}