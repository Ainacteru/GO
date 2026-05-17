#![no_std]
#![no_main]

use core::panic::PanicInfo;

use GO::actuators::servo;
use GO::actuators::servo::Servo;
use GO::pac::evsys::channel;
use atsamd_hal::delay::Delay;
use atsamd_hal::pwm::Channel;
use atsamd_hal::pwm::Pwm0;

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
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let servo1:Servo<bsp::Servo1Pwm> = Servo::new(pins.servo1.into());

    delay.delay_ms(500u32);

    loop {
        servo1.set_pos(&mut pwm0, 0);
        delay.delay_ms(500u32);
        servo1.set_pos(&mut pwm0, 50);
        delay.delay_ms(500u32);
        servo1.set_pos(&mut pwm0, 100);
        delay.delay_ms(500u32);
        servo1.set_pos(&mut pwm0, 150);
        delay.delay_ms(500u32);
        servo1.set_pos(&mut pwm0, 400);
        delay.delay_ms(500u32);
        servo1.set_pos(&mut pwm0, 250);
        delay.delay_ms(500u32);
        servo1.set_pos(&mut pwm0, 300);
        delay.delay_ms(500u32);

    }
}

#[panic_handler]
fn panic (_info: &PanicInfo) -> ! {
    uprintln!("\n");
    uprintln!("PANIC! {}", _info);
    loop {}
}