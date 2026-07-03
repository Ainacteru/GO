
use core::str::{Utf8Error, from_utf8};

use atsamd_hal::{ehal_async::spi::SpiBus, prelude::_atsamd_hal_embedded_hal_digital_v2_OutputPin};
use defmt::{info, warn};
use uom::si::{f32::Time, time::millisecond};
use crate::storage::{error::FlashError, flash::W25Q, flash_writer::RecordType::{BARO, EVENT, IMU, MESSAGE, OTHER}};

const CAPACITY: u32 = 2097152; // 2 * 1024 * 1024 == 2 MiB

pub struct FlashWriter <'a, 'b, SPI, CS> {
    flash: &'a mut W25Q<'b, SPI, CS>,
    cursor: u32,
    erased: u32
}

impl<'a, 'b, SPI, CS> FlashWriter<'a, 'b, SPI, CS>
where
    SPI: SpiBus<u8>,
    CS: _atsamd_hal_embedded_hal_digital_v2_OutputPin, 
{
    /// Return new flash writer and starting the next write where it was previously left off at
    pub async fn resume(w25: &'a mut W25Q<'b, SPI, CS>) -> Self {
        let mut writer = Self {
            flash: w25,
            cursor: 0x00,
            erased: 0
        };
        writer.find_cursor().await;

        writer
    }

    /// ERASES the chip or sector before returning
    pub async fn new(w25: &'a mut W25Q<'b, SPI, CS>) -> Self {
        let mut writer = Self {
            flash: w25,
            cursor: 0x00,
            erased: 0
        };
        writer.erase_sector(0x00).await;
        warn!("erasing sector!");
        writer.find_cursor().await;

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

    pub async fn erase_chip(&mut self) {
        self.flash.erase_chip().await;
    }

    pub async fn erase_sector(&mut self, sector: u32) {
        self.flash.erase_sector(sector).await;
    }

    
    pub async fn read(&mut self, addr: u32) -> Record {
        let mut header = [0u8; 6];  
        let mut message = [0u8; 255];
        self.flash.read(addr, &mut header).await;
        self.flash.read(addr + header.len() as u32, &mut message).await;

        Record::from_header(&header, message)
    }

    /// Looks for the next place to write by setting the cursor there
    async fn find_cursor(&mut self) {
        let mut current_addr: u32 = 0x00;
        let mut header = [0u8; 6];
        self.flash.read(current_addr, &mut header).await;

        while header[0] != 0xFF && current_addr < CAPACITY {
            current_addr += header.len() as u32 + header[5] as u32;
            self.flash.read(current_addr, &mut header).await;
        }

        self.cursor = current_addr;
        self.erased = self.cursor.div_ceil(4096) * 4096;
        info!("Flash writer continuing at {:#02x}", &self.cursor);

    }
}

pub struct Record {
    rtype: RecordType,
    timestamp: u32,
    length: u8,
    message: [u8; 255],
}

impl Record {
    pub fn from_header(header: &[u8], message: [u8; 255]) -> Self {
        Self {
            rtype: header[0].into(),
            timestamp: u32::from_le_bytes(header[1..5].try_into().unwrap()),
            length: header[5],
            message,
        }
    }

    pub fn new(rtype: RecordType, timestamp: u32, message: [u8; 255]) -> Self{
        Self {
            rtype,
            timestamp,
            length: message.len() as u8,
            message,
        }
    }

    pub fn get_record_type(&self) -> &str {
        self.rtype.into_str()
    }
    /// Returns the 
    pub fn get_timestamp(&self) -> Time {
        Time::new::<millisecond>(self.timestamp as f32)
    }
    pub fn get_message(&self) -> Result<&str, Utf8Error> {
        from_utf8(&self.message[..self.length as usize])
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum RecordType {
    MESSAGE = 0,
    IMU = 1,
    BARO = 2,
    EVENT = 3,
    OTHER = 255,
}

impl From<u8> for RecordType {
    
    fn from(value: u8) -> Self {
        match value {
            0 => MESSAGE,
            1 => IMU,
            2 => BARO,
            3 => EVENT,
            _ => OTHER,
        }
    }
}


impl RecordType {
    #[allow(clippy::wrong_self_convention)]
    fn into_str(&self) -> &str {
        match self {
            MESSAGE => "MESSAGE",
            IMU => "IMU",
            BARO => "BARO",
            EVENT => "EVENT",
            OTHER => "OTHER",
        }
    }
}