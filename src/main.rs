#![no_std]
#![no_main]


use atsamd_hal::{
    clock::GenericClockController, dmac::{DmaController, PriorityLevel}, fugit::RateExtU32, gpio::{Output, PA17, Pin}, pac::{Interrupt, NVIC, Peripherals, Sercom3, Tc4}, prelude::{_atsamd_hal_embedded_hal_digital_v2_OutputPin, _atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin}, sercom::Sercom4,
};
use block_device_adapters::BufStream;
use defmt::warn;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use embedded_fatfs::{FileSystem, FsOptions};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use go::{ Pins, communcation::{time_driver, usb::Usb}, peripherals::spi::Spi };
use sdspi::SdSpi;

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

    // Deselect the other SPI devices so they don't drive MISO
    let mut flash_cs = pins.flash_cs.into_push_pull_output();
    flash_cs.set_high().unwrap();
    let mut rf_cs = pins.rf_cs.into_push_pull_output();
    rf_cs.set_high().unwrap();

    // Setup DMA
    let dmac = DmaController::init(peripherals.dmac, &mut peripherals.pm);
    let mut dmac = dmac.into_future(Irqs);
    let channels = dmac.split();
    let chan0 = channels.0.init(PriorityLevel::Lvl0);
    let chan1 = channels.1.init(PriorityLevel::Lvl0);

    let mut spi = Spi::new((pins.sclk, pins.mosi, pins.miso), peripherals.sercom4, 400.kHz(), &mut clocks, &mut peripherals.pm).into_async(Irqs, (chan0, chan1));

    let mut cs = pins.sd_cs.into_push_pull_output();
    
    loop {
        match sdspi::sd_init(&mut spi, &mut cs).await {
            Ok(_) => break,
            Err(_) => { warn!("sd_init failed"); Timer::after_millis(10).await; }
        }
    }

    let dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();

    let mut sd = SdSpi::<_, _, aligned::A1>::new(dev, embassy_time::Delay);
    loop {
        if sd.init().await.is_ok() { break; }
        defmt::info!("retrying card init...");
        Timer::after_millis(10).await;
    }

    let inner = BufStream::<_, 512>::new(sd);
    let fs = FileSystem::new(inner, FsOptions::new()).await.unwrap();

    {
        let root = fs.root_dir();
        let mut file = root.create_file("test.txt").await.unwrap();
        file.write_all(b"hello\n").await.unwrap();
        file.flush().await.unwrap();
    }

    fs.unmount().await.unwrap();

    Timer::after_millis(2000).await;

    
}

fn enable_interrupts() {
    unsafe {
        NVIC::unmask(Interrupt::USB);
        NVIC::unmask(Interrupt::DMAC);
        NVIC::unmask(Interrupt::SERCOM4);
    }
}

#[embassy_executor::task]
async fn blink(mut pin: Pin<PA17, Output<atsamd_hal::gpio::PushPull>>) {
    loop {
        pin.toggle();
        Timer::after_millis(500).await;
    }
}
