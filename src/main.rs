#![no_std]
#![no_main]

use atsamd_hal::{
    clock::GenericClockController,
    dmac::{Ch0, channel::ReadyFuture, Channel, DmaController, PriorityLevel},
    gpio::{Output, PA17, Pin},
    pac::{Interrupt, NVIC, Peripherals, Sercom3, Tc4},
    prelude::_atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin,
    sercom::i2c::{self, I2cFuture},
};
use defmt::{info, println};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use go::{
    Pins, communcation::{time_driver, usb::Usb}, peripherals::i2c::I2c, sensors::{bmp, imu},
};
use static_cell::StaticCell;
use uom::si::{length::centimeter, pressure::pascal, thermodynamic_temperature::degree_fahrenheit};

atsamd_hal::bind_interrupts!(struct Irqs {
    SERCOM3 => atsamd_hal::sercom::i2c::InterruptHandler<Sercom3>;
    TC4 => atsamd_hal::timer::InterruptHandler<Tc4>;
    DMAC => atsamd_hal::dmac::InterruptHandler;
});

type I2cBus = Mutex<NoopRawMutex, I2cFuture<i2c::Config<go::I2cPads>, Channel<Ch0, ReadyFuture>>>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );

    let pins = Pins::new(peripherals.port);

    Usb::set_up(&mut clocks, &mut peripherals.pm, pins.usb_dm, pins.usb_dp, peripherals.usb);
    time_driver::init(peripherals.tc3, &mut peripherals.pm, &mut clocks);

    enable_interrupts();

    // Initialize DMA Controller
    let dmac = DmaController::init(peripherals.dmac, &mut peripherals.pm);
    let mut dmac = dmac.into_future(Irqs);
    let channels = dmac.split();
    let channel0 = channels.0.init(PriorityLevel::Lvl0);

    let i2c = I2c::new((pins.sda, pins.scl), peripherals.sercom3, &mut clocks, &mut peripherals.pm)
        .into_async(Irqs, channel0);

    // Shared I2C bus
    static I2C_BUS: StaticCell<I2cBus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    let mut imu = imu::Imu::new(I2cDevice::new(i2c_bus), Delay).await;
    let mut bmp = bmp::Bmp::new(I2cDevice::new(i2c_bus), Delay).await.unwrap();


    let led = pins.led.into_push_pull_output();
    spawner.spawn(blink(led).unwrap());

    Timer::after_millis(1000).await;

    loop {

        let mes = bmp.inner().read_latest_measurement().await.unwrap().into_uom();
        let temp = mes.temperature_celsius().get::<degree_fahrenheit>();
        let pres = mes.pressure_pascal();
        let alt = bmp.altitude().await.get::<centimeter>();

        info!("Temperature: {} F", temp);
        info!("pressure: {} Pa", pres.get::<pascal>());
        info!("altitude: {} cm", alt);

        println!("");

        let accel_data = imu.get_accel_data().await;
        info!("Accel x: {}", accel_data.x);
        info!("Accel y: {}", accel_data.y);
        info!("Accel z: {}", accel_data.z);

        let gyro_data = imu.get_gyro_data().await;
        info!("Gyro x: {}", gyro_data.x);
        info!("Gyro y: {}", gyro_data.y);
        info!("Gyro z: {}", gyro_data.z);

        println!("");
        Timer::after_millis(100).await;
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
