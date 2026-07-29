use atsamd_hal::{async_hal::interrupts::Binding, clock::GenericClockController, dmac::{Ch0, Channel, ReadyFuture}, pac, sercom::i2c::{self, I2cFuture, InterruptHandler}, time::Hertz};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use static_cell::StaticCell;

use crate::{I2cSercom, Scl, Sda, i2c_master};

type I2cBus = Mutex<NoopRawMutex, I2cFuture<i2c::Config<crate::I2cPads>, Channel<Ch0, ReadyFuture>>>;

static I2C_BUS: StaticCell<I2cBus> = StaticCell::new();

pub struct I2c {
    bus: &'static I2cBus,
}

impl I2c {
    #[allow(clippy::too_many_arguments)] // :3
    pub fn new<I>(
        clocks: &mut GenericClockController,
        baud: impl Into<Hertz>,
        sercom: I2cSercom,
        pm: &mut pac::Pm,
        sda: impl Into<Sda>,
        scl: impl Into<Scl>,
        irqs: I,
        dma_channel: Channel<Ch0, ReadyFuture>,
    ) -> Self
    where
        I: Binding<<I2cSercom as atsamd_hal::sercom::Sercom>::Interrupt, InterruptHandler<I2cSercom>>,
    {
        let i2c = i2c_master(clocks, baud, sercom, pm, sda, scl)
            .into_future(irqs)
            .with_dma_channel(dma_channel);

        Self {
            bus: I2C_BUS.init(Mutex::new(i2c)),
        }
    }

    pub fn bus(&self) -> &'static I2cBus {
        self.bus
    }
}
