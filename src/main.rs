#![no_std]
#![no_main]


use core::f32;

use atsamd_hal::{
    clock::GenericClockController, dmac::{DmaController, PriorityLevel}, fugit::RateExtU32, gpio::{Output, PA17, Pin}, pac::{Interrupt, NVIC, Peripherals, Sercom3, Tc4}, prelude::_atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin, sercom::Sercom4,
};
use defmt::{info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use go::{ Pins, communcation::{time_driver, usb::Usb}, control::kalman_filter::KalmanFilter, peripherals, sensors::{bmp::Bmp, imu::Imu} };
use libm::{asin, atan2f};
use micromath::{F32Ext, vector::F32x3};
use uom::si::length;

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

    let imu = Imu::new(I2cDevice::new(i2c.bus()), Delay).await.unwrap();
    let baro = Bmp::new(I2cDevice::new(i2c.bus()), Delay).await.unwrap();

    let mut kf = KalmanFilter::new(imu, baro);

    loop {
        kf.calc_orientation().await.unwrap();
        kf.calc_altitude().await.unwrap();

        let h = kf.altitude().get::<length::meter>();

        info!("height: {} meters", h);
    }

    // loop {

    //     kf.calc_atitude().await.unwrap();

    //     let q = kf.atitude();

    //     // let (roll, pitch, yaw) = kf.state().to_euler();
    //     // info!("r: {}", roll * 57.2958);
    //     // info!("p: {}", pitch * 57.2958);
    //     // info!("y: {}", yaw * 57.2958);

    //     let up = kf.atitude().conj().rotate(F32x3 { x: 0.0, y: 0.0, z: 1.0 });
    //     // let tilt_deg = libm::acosf(up.z.clamp(-1.0, 1.0)) * 180.0 / core::f32::consts::PI;


    //         // kf.imu_dat().await;
    //         // info!("q: {} {} {} {}\n", q.w(), q.x(), q.y(), q.z());

    //         info!("up.x: {}", up.x);
    //         info!("up.y: {}", up.y);
    //         info!("up.z: {}\n", up.z);

    //         const RAD: f32 = 180.0 / core::f32::consts::PI;
            
    //         let tilt   = libm::acosf(up.z.clamp(-1.0, 1.0)) * RAD;  // 0..180 off vertical
    //         let lean_x = libm::asinf(up.x.clamp(-1.0, 1.0)) * RAD;  // -90..+90 toward PCB normal
    //         let lean_y = libm::asinf(up.y.clamp(-1.0, 1.0)) * RAD;  // -90..+90 toward right

    //         info!("tilt: {}", &&tilt);
    //         info!("lean_x: {}", &&lean_x);
    //         info!("lean_y: {}\n", &&lean_y);

            

    //     Timer::after_millis(10).await;
    // }

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
