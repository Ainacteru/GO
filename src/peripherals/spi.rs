use crate::{self as go, Miso, Mosi, Sclk, SpiSercom};
use atsamd_hal::{
    async_hal::interrupts::Binding, clock::GenericClockController, pac, sercom::{self, spi::{self, InterruptHandler, SpiFuture}}, time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use static_cell::StaticCell;

type SpiBus = Mutex<NoopRawMutex, SpiFuture<spi::Config<crate::SpiPads, sercom::spi::Master, sercom::spi::EightBit>, sercom::spi::Duplex>>;

static SPI_BUS: StaticCell<SpiBus> = StaticCell::new();

pub struct Spi {
    bus: &'static SpiBus,
}

impl Spi {
    pub fn new<I>(
        pins: (impl Into<Sclk>, impl Into<Mosi>, impl Into<Miso>),
        sercom: go::SpiSercom,
        baud: Hertz,
        clocks: &mut GenericClockController,
        pm: &mut pac::Pm,
        irqs: I,
    ) -> Self 
    where
        I: Binding<<SpiSercom as atsamd_hal::sercom::Sercom>::Interrupt, InterruptHandler<SpiSercom>>,
    {
        
        let spi = go::spi_master(clocks, baud, sercom, pm, pins.0, pins.1, pins.2)
        .into_future(irqs);
        Self {
            bus: SPI_BUS.init(Mutex::new(spi)),
        }
    }

    pub fn bus(&self) -> &'static SpiBus {
        self.bus
    }

}
