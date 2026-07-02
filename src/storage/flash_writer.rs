use atsamd_hal::{ehal_async::spi::SpiBus, prelude::_atsamd_hal_embedded_hal_digital_v2_OutputPin} ;

use crate::storage::{error::FlashError, flash::W25Q};

const CAPACITY: u32 = 2097152; // 2 * 1024 * 1024 == 2 MiB

pub struct FlashWriter <'a, 'b, SPI, CS> {
    flash: &'a mut W25Q<'b, SPI, CS>,
    cursor: u32,
    erased: u32
}

/// Create new flash writer and starting the next write where it was previously left off at
impl<'a, 'b, SPI, CS> FlashWriter<'a, 'b, SPI, CS>
where
    SPI: SpiBus<u8>,
    CS: _atsamd_hal_embedded_hal_digital_v2_OutputPin, 
{
    pub async fn new(w25: &'a mut W25Q<'b, SPI, CS>) -> Self {
        let mut writer = Self {
            flash: w25,
            cursor: 0x00,
            erased: 0
        };
        writer.resume().await;

        writer
    }

    /// Write a 6 byte header containing type, timestamp, and msg length, then write the message
    pub async fn write(&mut self, record_type: RecordType, data: &[u8]) -> Result<(), FlashError> {
        if data.len() > 255 { 
            return Err(FlashError::RecordTooLarge)
        }


        let timestamp = embassy_time::Instant::now().as_millis() as u32;
        let mut header = [0u8; 6]; // contains log type, time stamp and data

        header[0] = record_type as u8;
        header[1..5].copy_from_slice(&timestamp.to_le_bytes()); // will have to read timestamp as little endian in decoder
        header[5] = data.len() as u8;

        let end = self.cursor + header.len() as u32 + data.len() as u32;

        if end > CAPACITY {
            return Err(FlashError::OutOfBounds)
        }

        while self.erased < end {
            let sector = self.erased / 4096;
            self.flash.erase_sector(sector).await;
            self.erased += 4096;
        }
        self.flash.write(self.cursor, &header).await;
        self.cursor += header.len() as u32;

        self.flash.write(self.cursor, data).await;
        self.cursor += data.len() as u32;
        Ok(())
    }

    pub async fn wipe_flash(&mut self) {
        self.flash.erase_chip().await;
    }

    pub async fn read(&mut self, addr: u32, out: &mut [u8]) {
        self.flash.read(addr, out).await;
    }

    async fn resume(&mut self) {
        let mut current_addr: u32 = 0x00;
        let mut header = [0u8; 6];
        self.read(current_addr, &mut header).await;

        while header[0] != 0xFF && current_addr < CAPACITY {
            current_addr += header.len() as u32 + header[5] as u32;
            self.read(current_addr, &mut header).await;
        }

        self.cursor = current_addr;
        self.erased = self.cursor.div_ceil(4096) * 4096;
    }
}

#[repr(u8)]
pub enum RecordType {
    MESSAGE = 0,
    IMU = 1,
    BARO = 2,
    EVENT = 3,
}