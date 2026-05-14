#![no_std]
#![no_main]

use atsamd_hal::delay::Delay;
use atsamd_hal::gpio;
use atsamd_hal::pwm::Channel;
use atsamd_hal::pwm::Pwm0;
use panic_halt as _;

use bsp::hal;
use bsp::pac;
use samdhal::uprintln;
use samdhal as bsp;

use bsp::{entry};
use hal::clock::GenericClockController;
use hal::prelude::*;
use pac::{CorePeripherals, Peripherals};

use samdhal::communcation::usb;
use samdhal::actuators::servo;

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
    let mut pwm0 = Pwm0::new(&clocks.tcc0_tcc1(&glck0).unwrap(), 1.kHz(), peripherals.tcc0, &mut peripherals.pm);
    let max_duty = pwm0.get_max_duty();

    let _pa21:bsp::RgbBluePwm = pins.rgb_blue.into();
    let mut led = pins.led.into_push_pull_output();
    let mut delay = Delay::new(core.SYST, &mut clocks);




    loop {

        for i in 1..=100 {
            pwm0.set_duty(Channel::_0, (max_duty * (100 - i) * (100 - i)) / 10000);
            delay.delay_ms(10u32);
            uprintln!("{}", (max_duty * (100 - i) * (100 - i)) / 10000);
        }
        for i in (1..=100).rev() {
            pwm0.set_duty(Channel::_0, (max_duty * (100 - i) * (100 - i)) / 10000);
            delay.delay_ms(10u32);
            uprintln!("{}", (max_duty * (100 - i) * (100 - i)) / 10000);
        }

        // delay.delay_ms(500u32);
        // pwm0.set_duty(Channel::_3, max_duty / 2);
        // uprintln!("maxduty {}", max_duty);
        // delay.delay_ms(500u32);
        // pwm0.set_duty(Channel::_3, max_duty);
        // uprintln!("maxduty {}", max_duty);
    }
}