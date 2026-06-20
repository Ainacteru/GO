use crate::{self as go, Scl, Sda};
use atsamd_hal::{
    clock::GenericClockController,
    dmac::{AnyChannel, ReadyFuture},
    fugit::RateExtU32,
    pac,
    sercom::i2c::{self, I2cFuture, InterruptHandler},
};

pub struct I2c {
    pub i2c: go::I2c,
}

impl I2c {
    pub fn new(
        pins: (impl Into<Sda>, impl Into<Scl>),
        sercom: go::I2cSercom,
        clocks: &mut GenericClockController,
        pm: &mut pac::Pm,
    ) -> Self {
        Self {
            i2c: go::i2c_master(clocks, 100_u32.kHz(), sercom, pm, pins.0, pins.1),
        }
    }

    pub fn into_async<I, D>(
        self,
        irqs: I,
        dma_channel: D,
    ) -> I2cFuture<i2c::Config<go::I2cPads>, D>
    where
        I: atsamd_hal::async_hal::interrupts::Binding<
            <go::I2cSercom as atsamd_hal::sercom::Sercom>::Interrupt,
            InterruptHandler<go::I2cSercom>,
        >,
        D: AnyChannel<Status = ReadyFuture>,
    {
        self.i2c.into_future(irqs).with_dma_channel(dma_channel)
    }
}
