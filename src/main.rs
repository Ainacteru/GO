#![no_std]
#![no_main]

use core::panic::PanicInfo;

use go::Led;
use go::RgbRed;
use go::RgbRedPwm;
use go::actuators::servo;
use go::actuators::servo::Servo;
use go::indicators::Leds::rgb_leds;
use go::indicators::Leds::rgb_leds::RgbLed;
use go::pac::evsys::channel;
use atsamd_hal::delay::Delay;
use atsamd_hal::pwm::Channel;
use atsamd_hal::pwm::Pwm0;

use go as bsp;
use atsamd_hal::pwm::Pwm2;
use bsp::hal;
use bsp::pac;

use bsp::{entry};
use hal::clock::GenericClockController;
use hal::prelude::*;
use pac::{CorePeripherals, Peripherals};

use go::{uprintln, uprint};
use go::communcation::usb;

#[entry]
fn main() -> ! {
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
    let mut pwm0 = Pwm0::new(&clocks.tcc0_tcc1(&glck0).unwrap(), 50.Hz(), peripherals.tcc0, &mut peripherals.pm);
    let mut pwm2 = Pwm2::new(&clocks.tcc2_tc3(&glck0).unwrap(), 50.Hz(), peripherals.tcc2, &mut peripherals.pm);

    
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let servo1:Servo<bsp::Servo1Pwm> = Servo::new(pins.servo1.into());
    let servo2:Servo<bsp::Servo2Pwm> = Servo::new(pins.servo2.into());
    let servo3:Servo<bsp::Servo3Pwm> = Servo::new(pins.servo3.into());

    let red: RgbLed::<bsp::RgbBluePwm> = rgb_leds::RgbLed::new(pins.rgb_blue.into());
    delay.delay_ms(500u32);
    usb::set_input_ready();
    uprint!("starting");


    loop {
        red.set_brightness(&mut pwm0, 50);
        delay.delay_ms(100u32);
        red.set_brightness(&mut pwm0, 100);
        delay.delay_ms(100u32);
    }
}
#[panic_handler]
fn panic (_info: &PanicInfo) -> ! {
    let mut peripherals = unsafe { Peripherals::steal() };
    let pins = bsp::Pins::new(peripherals.port);
    let mut led: Led = pins.led.into();
    let mut red: RgbRed = pins.rgb_red.into();
    unsafe { cortex_m::interrupt::enable() };

    uprintln!("PANIC! {}", _info);

    red.set_high();

    loop {
        led.toggle();

        cortex_m::asm::delay(6_500_000); // ~1s at 8MHz
    }
}