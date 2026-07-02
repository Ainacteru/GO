#![no_std]
#![no_main]

use core::str::from_utf8;

use atsamd_hal::{
    clock::GenericClockController, dmac::{DmaController, PriorityLevel}, fugit::RateExtU32, gpio::{Output, PA17, Pin}, pac::{Interrupt, NVIC, Peripherals, Sercom3, Tc4}, prelude::{_atsamd_hal_embedded_hal_digital_v2_OutputPin, _atsamd_hal_embedded_hal_digital_v2_ToggleableOutputPin}, sercom::Sercom4,
};
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_time::Timer;
use go::{ Pins, communcation::{time_driver, usb::Usb}, storage::{flash::W25Q, flash_writer::{FlashWriter, RecordType}} };

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
    clocks.tcc2_tc3(&gclk0).expect("no tcc2"); // keep bc you have to set up tc3
    time_driver::init(peripherals.tc3, &mut peripherals.pm);

    enable_interrupts();

    let led = pins.led.into_push_pull_output();
    spawner.spawn(blink(led).unwrap());

    // Deselect the other SPI devices so they don't drive MISO
    let mut sd_cs = pins.sd_cs.into_push_pull_output();
    sd_cs.set_high().unwrap();
    let mut rf_cs = pins.rf_cs.into_push_pull_output();
    rf_cs.set_high().unwrap();

    // WP (FLASH_EN) high = writes allowed
    let mut flash_wp = pins.flash_en.into_push_pull_output();
    flash_wp.set_high().unwrap();

    // Setup DMA
    let dmac = DmaController::init(peripherals.dmac, &mut peripherals.pm);
    let mut dmac = dmac.into_future(Irqs);
    let channels = dmac.split();
    let chan0 = channels.0.init(PriorityLevel::Lvl0);
    let chan1 = channels.1.init(PriorityLevel::Lvl0);

    let mut spi = go::spi_master(
        &mut clocks,
        100.kHz(),
        peripherals.sercom4,
        &mut peripherals.pm,
        pins.sclk,
        pins.mosi,
        pins.miso,
    )
    .into_future(Irqs)
    .with_dma_channels(chan0, chan1);

    let mut cs = pins.flash_cs.into_push_pull_output();

    Timer::after_millis(2000).await;

    let mut flash = W25Q::new(&mut spi, &mut cs).await;

    let mut writer = FlashWriter::new(&mut flash).await;

    writer.write(RecordType::MESSAGE, "hello!!!".as_bytes()).await.unwrap();
    // info!("wrote message");

    loop {
        let mut buf = [0u8; 8];
        writer.read(0x0, &mut buf).await;
        match from_utf8(&buf) {
            Ok(msg) => info!("read back: {}", msg),
            Err(_) => warn!("non-utf8: {:02x}", buf),
        }
        Timer::after_millis(500).await;
    }
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
