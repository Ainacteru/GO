#![no_std]
#![no_main]


use atsamd_hal::{
    clock::GenericClockController, dmac::{DmaController, PriorityLevel}, fugit::RateExtU32, gpio::{Output, PA17, Pin}, pac::{Interrupt, NVIC, Peripherals, Sercom3, Tc4}, prelude::_atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin, sercom::Sercom4,
};
use defmt::{info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use go::{ Pins, communcation::{time_driver, usb::Usb}, peripherals, sensors::imu::Imu };
use uom::si::thermodynamic_temperature::degree_fahrenheit;

atsamd_hal::bind_interrupts!(struct Irqs {
    SERCOM3 => atsamd_hal::sercom::i2c::InterruptHandler<Sercom3>;
    TC4 => atsamd_hal::timer::InterruptHandler<Tc4>;
    DMAC => atsamd_hal::dmac::InterruptHandler;
    SERCOM4 => atsamd_hal::sercom::spi::InterruptHandler<Sercom4>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );
    let gclk0 = clocks.gclk0();
    let pins = Pins::new(peripherals.port);

    Usb::set_up(&mut clocks, &mut peripherals.pm, pins.usb_dm, pins.usb_dp, peripherals.usb);
    clocks.tcc2_tc3(&gclk0).expect("no tcc2"); // keep bc you have to set up tc3 for embassy
    time_driver::init(peripherals.tc3, &mut peripherals.pm);

    enable_interrupts();

    let led = pins.led.into_push_pull_output();
    spawner.spawn(blink(led).unwrap());

    let dmac = DmaController::init(peripherals.dmac, &mut peripherals.pm);
    let mut dmac = dmac.into_future(Irqs);
    let channels = dmac.split();
    let channel0 = channels.0.init(PriorityLevel::Lvl0);

    let i2c = peripherals::i2c::I2c::new(&mut clocks, 400.kHz(), peripherals.sercom3, &mut peripherals.pm, pins.sda, pins.scl, Irqs, channel0);

    Timer::after_secs(2).await;

    let mut imu = Imu::new(I2cDevice::new(i2c.bus()), Delay).await.unwrap();

    loop {
        let accel = imu.get_accel_data().await.unwrap();
        info!("Accel xyz: {}, {}, {}", accel.x, accel.y, accel.z);

        let gyro = imu.get_gyro_data().await.unwrap();
        info!("Gyro  xyz: {}, {}, {}", gyro.x, gyro.y, gyro.z);

        let temp = imu.get_temp_data().await.unwrap().get::<degree_fahrenheit>();
        info!("Temp:      {}F", temp);

        Timer::after_millis(250).await;
    }

}

fn enable_interrupts() {
    unsafe {
        NVIC::unmask(Interrupt::USB);
        NVIC::unmask(Interrupt::DMAC);
        NVIC::unmask(Interrupt::SERCOM4);
        NVIC::unmask(Interrupt::SERCOM3);
    }
}

#[embassy_executor::task]
async fn blink(mut pin: Pin<PA17, Output<atsamd_hal::gpio::PushPull>>) {
    loop {
        pin.toggle();
        Timer::after_millis(500).await;
    }
}
