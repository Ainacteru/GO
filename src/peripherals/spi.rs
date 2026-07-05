use crate::{self as go, Miso, Mosi, Sclk};
use atsamd_hal::{
    clock::GenericClockController, dmac::{AnyChannel, ReadyFuture}, pac, sercom::spi::{self, InterruptHandler, SpiFuture}, time::Hertz,
};

pub struct Spi {
    spi: go::Spi,
}

impl Spi {
    pub fn new(
        pins: (impl Into<Sclk>, impl Into<Mosi>, impl Into<Miso>),
        sercom: go::SpiSercom,
        baud: Hertz,
        clocks: &mut GenericClockController,
        pm: &mut pac::Pm,
    ) -> Self {
        Self {
            spi: go::spi_master(clocks, baud, sercom, pm, pins.0, pins.1, pins.2),
        }
    }

    pub fn into_async<I, R, T>(self, iqrs: I, dma_channel: (R, T)) -> SpiFuture<spi::Config<go::SpiPads>, spi::Duplex, R, T>
        where
        I: atsamd_hal::async_hal::interrupts::Binding<
            <go::SpiSercom as atsamd_hal::sercom::Sercom>::Interrupt,
            InterruptHandler<go::SpiSercom>,
        >,
        R: AnyChannel<Status = ReadyFuture>,
        T: AnyChannel<Status = ReadyFuture>,
    {
        self.spi.into_future(iqrs).with_dma_channels(dma_channel.0, dma_channel.1)
    }

    /// Async SPI without DMA. Commands use full-duplex word-by-word transfers,
    /// which frame correctly for SD/flash command bytes (the DMA path does a
    /// TX-only transfer that can misframe single commands).
    pub fn into_async_nodma<I>(self, irqs: I) -> SpiFuture<spi::Config<go::SpiPads>, spi::Duplex>
    where
        I: atsamd_hal::async_hal::interrupts::Binding<
            <go::SpiSercom as atsamd_hal::sercom::Sercom>::Interrupt,
            InterruptHandler<go::SpiSercom>,
        >,
    {
        self.spi.into_future(irqs)
    }

}
