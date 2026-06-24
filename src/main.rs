#![no_std]
#![no_main]

use atsamd_hal::{
    clock::{GenericClockController, Tcc2Tc3Clock}, dmac::{Ch0, Channel, DmaController, PriorityLevel, channel::ReadyFuture}, fugit::RateExtU32, gpio::{Output, PA17, Pin}, pac::{Interrupt, NVIC, Peripherals, Sercom3, Tc4}, prelude::{_atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin, _embedded_hal_Pwm}, pwm::{self, Pwm2}, sercom::i2c::{self, I2cFuture},
};
use defmt::{info, println};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use go::{ Buzzer as buzzzzz, Pins, communcation::{time_driver, usb::Usb}, indicators::buzzer::Buzzer, pac::Tcc2, peripherals::i2c::I2c, sensors::{bmp, imu},
};
use static_cell::StaticCell;
use uom::si::{length::centimeter, pressure::pascal, thermodynamic_temperature::degree_fahrenheit};

atsamd_hal::bind_interrupts!(struct Irqs {
    SERCOM3 => atsamd_hal::sercom::i2c::InterruptHandler<Sercom3>;
    TC4 => atsamd_hal::timer::InterruptHandler<Tc4>;
    DMAC => atsamd_hal::dmac::InterruptHandler;
});

// type I2cBus = Mutex<NoopRawMutex, I2cFuture<i2c::Config<go::I2cPads>, Channel<Ch0, ReadyFuture>>>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );
    let glck0 = clocks.gclk0();

    let pins = Pins::new(peripherals.port);

    Usb::set_up(&mut clocks, &mut peripherals.pm, pins.usb_dm, pins.usb_dp, peripherals.usb);
    let tcc2_tc3_clock = clocks.tcc2_tc3(&glck0).expect("no tcc2");
    time_driver::init(peripherals.tc3, &mut peripherals.pm);

    enable_interrupts();

    let led = pins.led.into_push_pull_output();
    spawner.spawn(blink(led).unwrap());

    let b: buzzzzz = pins.buzzer.into();

    let tcc2 = Pwm2::new(&tcc2_tc3_clock, 440.Hz(), peripherals.tcc2, &mut peripherals.pm);
    let mut buz = Buzzer::new(b, tcc2);
    
    buz.set_volume(0);

    Timer::after_millis(1000).await;
    loop {
        use go::indicators::buzzer::Note::*;

        // "Hot cross buns"
        buz.set_volume(20);
        buz.set_note(B4); Timer::after_millis(500).await;
        buz.set_note(A4); Timer::after_millis(500).await;
        buz.set_note(G4); Timer::after_millis(500).await;
        buz.set_volume(0); Timer::after_millis(500).await;



        // "Hot cross buns"
        buz.set_volume(20);
        buz.set_note(B4); Timer::after_millis(500).await;
        buz.set_note(A4); Timer::after_millis(500).await;
        buz.set_note(G4); Timer::after_millis(500).await;
        buz.set_volume(0); Timer::after_millis(500).await;

        // "One a penny, two a penny"
        buz.set_volume(20); 
        buz.set_note(G4); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20);
        buz.set_note(A4); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20); Timer::after_millis(200).await;
        buz.set_volume(0); Timer::after_millis(50).await;
        buz.set_volume(20); Timer::after_millis(200).await;

        // "Hot cross buns"
        buz.set_note(B4); Timer::after_millis(500).await;
        buz.set_note(A4); Timer::after_millis(500).await;
        buz.set_note(G4); Timer::after_millis(500).await;
        buz.set_volume(0); Timer::after_millis(500).await;

        Timer::after_millis(1000).await;
    }
}

fn enable_interrupts() {
    unsafe {
        NVIC::unmask(Interrupt::USB);
    }
}

#[embassy_executor::task]
async fn blink(mut pin: Pin<PA17, Output<atsamd_hal::gpio::PushPull>>) {
    loop {
        pin.toggle();
        Timer::after_millis(500).await;
    }
}
