#![no_std]
#![no_main]

use atsamd_hal::{
    clock::GenericClockController, dmac::{DmaController, PriorityLevel}, fugit::RateExtU32, gpio::{Output, PA17, Pin}, pac::{CorePeripherals, Interrupt, NVIC, Peripherals, Sercom3, Tc4}, prelude::_atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin, sercom::i2c, timer::TimerCounter
};
use bmp390_rs::typestate::Bmp390Builder;
use defmt::{info};

use embassy_executor::Spawner;
use embassy_time::Timer;
use go::{Pins, communcation::{time_driver, usb::Usb}, peripherals::i2c::I2c, sensors::bmp::{self, Bmp}};
use uom::si::{f32::{Length, Pressure, ThermodynamicTemperature}, length::{centimeter, meter}, pressure::pascal, thermodynamic_temperature::{degree_celsius, degree_fahrenheit}};

atsamd_hal::bind_interrupts!(struct Irqs {
    SERCOM3 => atsamd_hal::sercom::i2c::InterruptHandler<Sercom3>;
    TC4 => atsamd_hal::timer::InterruptHandler<Tc4>;
    DMAC => atsamd_hal::dmac::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );

    let pins = Pins::new(peripherals.port);

    Usb::set_up(&mut clocks, &mut peripherals.pm, pins.usb_dm, pins.usb_dp, peripherals.usb);
    // timer::set_up(&mut clocks, peripherals.tc3, &mut peripherals.pm);
    time_driver::init(peripherals.tc3, &mut peripherals.pm, &mut clocks);

    enable_interrupts();
        
    let gclk0 = clocks.gclk0();

        // Initialize DMA Controller
    let dmac = DmaController::init(peripherals.dmac, &mut peripherals.pm);

    // Turn dmac into an async controller
    let mut dmac = dmac.into_future(Irqs);
    // Get individual handles to DMA channels
    let channels = dmac.split();

    // Initialize DMA Channel 0
    let channel0 = channels.0.init(PriorityLevel::Lvl0);

    let delay = TimerCounter::tc4_(&clocks.tc4_tc5(&gclk0).unwrap(), peripherals.tc4, &mut peripherals.pm).into_future(Irqs);

    let i2c = I2c::new((pins.sda, pins.scl), peripherals.sercom3, &mut clocks, &mut peripherals.pm).into_async(Irqs, channel0);


    // let mut bmp = Bmp390Builder::new()
    //     .use_i2c(i2c, bmp390_rs::SdoPinState::High)
    //     .enable_temperature()
    //     .enable_pressure()
    //     .into_normal()
    //     .build(bmp390_rs::ResetPolicy::Soft, delay).await.unwrap();

    let mut bmp =  Bmp::new(i2c, delay).await.unwrap();

    let led = pins.led.into_push_pull_output();

    spawner.spawn(blink(led).unwrap());

    Timer::after_millis(100).await;
    loop {
        let mes = bmp.inner().read_latest_measurement().await.unwrap().into_uom();
        let temp = mes.temperature_celsius().get::<degree_fahrenheit>();
        let pres = mes.pressure_pascal();
        let alt = bmp.altitude().await.get::<centimeter>();

        info!("Temperature: {} F", temp);
        info!("pressure: {} Pa", pres.get::<pascal>());
        info!("altitude: {} cm", alt);
        Timer::after_millis(100).await;
    }
}

fn enable_interrupts() {
    unsafe {
        NVIC::unmask(Interrupt::USB);
        NVIC::unmask(Interrupt::TC3);
    }
}

#[embassy_executor::task]
async fn blink(mut pin: Pin<PA17, Output<atsamd_hal::gpio::PushPull>>) {
    loop {
        pin.toggle();
        Timer::after_millis(500).await;
    }
}
