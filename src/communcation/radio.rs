use atsamd_hal::{ehal::{digital::{InputPin, OutputPin}, spi::Operation}, ehal_async::{delay::DelayNs, spi::SpiDevice}};

pub struct Radio<SPI, D> {
    spi: SPI,
    pins: RadioPins,
    delay: D
}
pub struct RadioPins {
    pub nrst: RfNrst,
    pub busy: RfBusy,
    pub int: RfInt,
}

impl<SPI, D> Radio<SPI, D>
where
    SPI: SpiDevice,
    D: DelayNs,
{
    pub async fn new(spi: SPI, pins: RadioPins, delay: D) -> Result<Self, RadioError> {
        let mut radio = Self {
            spi,
            pins,
            delay,
        };
        //reset
        //wait
        //standby
        //dio2 as rf switch
        //voltage reg mode
        //calibration
        //fallback

        radio.restart().await?;
        radio.busy().await?;
        radio.set_standby().await?;
        radio.set_dio2().await?;
        

        Ok(radio)
    }

    async fn busy(&mut self) -> Result<(), RadioError> {
        while self.pins.busy.is_high().map_err(|_| RadioError::Peripherals)? {
            self.delay.delay_ms(10).await;
        }
        Ok(())
    }

    async fn restart(&mut self) -> Result<(), RadioError> {
        self.pins.nrst.set_low().map_err(|_| RadioError::Peripherals)?;
        self.delay.delay_us(100).await;
        self.pins.nrst.set_high().map_err(|_| RadioError::Peripherals)?;
        Ok(())
    }

    async fn set_dio2(&mut self) -> Result<(), RadioError> { 
        self.spi_write(SetDIO2, &[0x00]).await.map_err(|_| RadioError::SPI)?;
        Ok(())
    }

    async fn set_standby(&mut self) -> Result<(), RadioError> {
        self.spi_write(SetStandby, &[0x01]).await.map_err(|_| RadioError::SPI)?;
        Ok(())
    }

    async fn spi_write(&mut self, command: RadioCommands, parameter: &[u8]) -> Result<(), RadioError> {
        self.busy().await?;

        self.spi.transaction(&mut [
            Operation::Write(&[command as u8]),
            Operation::Write(parameter)
        ]).await.map_err(|_| RadioError::SPI)?;

        self.busy().await?;
        Ok(())
    }
}

#[repr(u8)]
enum RadioCommands {
    SetStandby = 0x80,
    SetDIO2 = 0x9D,
}


use thiserror::Error;

use crate::{RfBusy, RfInt, RfNrst, communcation::radio::RadioCommands::{SetDIO2, SetStandby}};

#[derive(Error, Debug)]
pub enum RadioError {
    #[error("Power Error")]
    Power,
    #[error("Initialization Error")]
    Initialization,
    #[error("SPI Error")]
    SPI,
    #[error("Peripheral Error")]
    Peripherals,
}