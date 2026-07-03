use core::cmp::min;

use atsamd_hal::{ehal_async::spi::SpiBus, prelude::_atsamd_hal_embedded_hal_digital_v2_OutputPin};
use defmt::info;
use embassy_time::Timer;

pub struct W25Q<'a, SPI, CS> {
    spi: &'a mut SPI,
    cs: &'a mut CS,
}

impl<'a, SPI, CS> W25Q<'a, SPI, CS>
where
    SPI: SpiBus<u8>,
    CS: _atsamd_hal_embedded_hal_digital_v2_OutputPin,
{
    pub async fn new(spi: &'a mut SPI, cs: &'a mut CS) -> Self {
        let mut flash = Self { 
            spi, 
            cs, 
        };


        let mut buf: [u8; 6] = [0x4B, 0, 0, 0, 0, 0];
        flash.xfer(&mut buf).await;

        // assert_eq!(flash.jedec_id().await, [0xef, 0x40, 0x15]);
        info!("Found flash chip with id: {:#02x}", &buf[5]);

        flash
    }

    async fn xfer(&mut self, buf: &mut [u8]) {
        self.cs.set_low().ok();
        self.spi.transfer_in_place(buf).await.ok();
        self.cs.set_high().ok();
    }

    pub async fn jedec_id(&mut self) -> [u8; 3] {
        let mut buf = [0x9F, 0, 0, 0];
        self.xfer(&mut buf).await;
        [buf[1], buf[2], buf[3]]
    }

    pub async fn read_sr1(&mut self) -> u8 {
        let mut buf = [0x05, 0x00];
        self.xfer(&mut buf).await;
        buf[1]
    }

    pub async fn write_enable(&mut self) {
        let mut buf = [0x06];
        self.xfer(&mut buf).await;
    }

    pub async fn wait_busy(&mut self) {
        // Busy bit is SR1 bit 0
        while self.read_sr1().await & 0x01 != 0 {
            Timer::after_micros(50).await;
        }
    }

    pub async fn erase_sector(&mut self, sector: u32) {
        let addr = sector * 4096;
        self.write_enable().await;
        let mut buf = [
            0x20,
            (addr >> 16) as u8,
            (addr >> 8) as u8,
            addr as u8,
        ];
        self.xfer(&mut buf).await;
        self.wait_busy().await;
    }

    pub async fn erase_chip(&mut self) {
        self.write_enable().await;

        self.xfer(&mut [0xC7]).await;
        self.wait_busy().await;
    }

    async fn write_page(&mut self, addr: u32, data: &[u8]) {
        self.write_enable().await;
        // command(4) + data
        let mut buf = [0u8; 4 + 256];
        buf[0] = 0x02;
        buf[1] = (addr >> 16) as u8;
        buf[2] = (addr >> 8) as u8;
        buf[3] = addr as u8;
        buf[4..4 + data.len()].copy_from_slice(data);
        self.xfer(&mut buf[..4 + data.len()]).await;
        self.wait_busy().await;
    }

    pub async fn write(&mut self, addr: u32, data: &[u8]) {
        let mut data = data;
        let mut addr = addr;
        while !data.is_empty()  {
            let page_offset: usize = (addr % 256).try_into().unwrap();
            let space_in_page: usize = 256 - page_offset;
            let chunk_len: usize = min(space_in_page, data.len());

            self.write_page(addr, &data[..chunk_len]).await;

            addr += chunk_len as u32;
            data = &data[chunk_len..];
        }
    }

    pub async fn read(&mut self, addr: u32, out: &mut [u8]) {
        let mut buf = [0u8; 4];

        buf[0] = 0x03;
        buf[1] = (addr >> 16) as u8;
        buf[2] = (addr >> 8) as u8;
        buf[3] = addr as u8;

        self.cs.set_low().ok();
        self.spi.transfer_in_place(&mut buf).await.unwrap();
        self.spi.transfer_in_place(out).await.unwrap();
        self.cs.set_high().ok();
    }
}