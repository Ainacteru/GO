#![no_std]
#![no_main]

use GO::actuators::servo;
use GO::actuators::servo::Servos;
use atsamd_hal::delay::Delay;
use atsamd_hal::pwm::Pwm0;
use panic_halt as _;

use GO as bsp;
use bsp::hal;
use bsp::pac;

use bsp::{entry};
use hal::clock::GenericClockController;
use hal::prelude::*;
use pac::{CorePeripherals, Peripherals};

use GO::{uprintln, uprint};
use GO::communcation::usb;

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

    // usb::set_up(
    //     peripherals.usb,
    //     &mut clocks,
    //     &mut peripherals.pm,
    //     pins.usb_dm,
    //     pins.usb_dp,
    //     &mut core.NVIC,
    // );


    let glck0 = clocks.gclk0();
    let mut pwm0 = Pwm0::new(&clocks.tcc0_tcc1(&glck0).unwrap(), 50.Hz(), peripherals.tcc0, &mut peripherals.pm);
    let mut servo = servo::Servo::new(Servos::Servo1, pwm0, pins);
    //let mut led = pins.led.into_push_pull_output();
    let mut delay = Delay::new(core.SYST, &mut clocks);




    loop {
        servo.set_position(200);
        delay.delay_ms(500u32);
        servo.set_position(0);

    }
}