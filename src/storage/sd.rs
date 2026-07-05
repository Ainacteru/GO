use atsamd_hal::ehal_async::spi::SpiDevice;
use block_device_adapters::BufStream;
use defmt::warn;
use embassy_time::{Delay, Duration, Instant, Timer};
use embedded_fatfs::{File, FileSystem, FsOptions, LossyOemCpConverter, NullTimeProvider};
use embedded_io_async::{Read, Seek, SeekFrom, Write};
use sdspi::SdSpi;

use crate::storage::error::SDCardError;

/// The raw SD block device
type SDDev<SPI> = SdSpi<SPI, Delay, aligned::A1>;

/// A mounted FAT filesystem 
type SDFs<SPI> = FileSystem<BufStream<SDDev<SPI>, 512>, NullTimeProvider, LossyOemCpConverter>;

/// An open file on the mounted filesystem
type SDFile<'a, SPI> = File<'a, BufStream<SDDev<SPI>, 512>, NullTimeProvider, LossyOemCpConverter>;

pub struct SDCard<SPI: SpiDevice> {
    card_state: CardState<SPI>,
}

impl<SPI: SpiDevice> SDCard<SPI> {
    /// Returns a new sd card struct, but DOES NOT start the file system.
    pub async fn new(dev: SPI) -> Result<Self, SDCardError> {
        let mut sdcard = SdSpi::new(dev, Delay);

        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        // 5 sec timeout on card init
        loop {
            if start.elapsed() >= timeout {
                return Err(SDCardError::Timeout);
            }
            match sdcard.init().await {
                Ok(_) => break,
                Err(_) => {
                    warn!("card init retry...");
                    Timer::after_millis(100).await;
                }
            }
        }
        warn!("card initialized");

        Ok(Self {
            card_state: CardState::Raw(sdcard),
        })
    }

    /// Mount filesystem
    pub async fn init_fs(&mut self) -> Result<(), SDCardError> {
        let state = core::mem::replace(&mut self.card_state, CardState::Empty);

        match state {
            CardState::Raw(sd) => {
                let inner = BufStream::<_, 512>::new(sd);
                let fs = FileSystem::new(inner, FsOptions::new())
                    .await
                    .map_err(|_| SDCardError::Io)?;
                self.card_state = CardState::Mounted(fs);
                Ok(())
            }

            CardState::Mounted(fs) => {
                self.card_state = CardState::Mounted(fs);
                Ok(())
            }
            CardState::Empty => Err(SDCardError::NoSDCard),
        }
    }

    /// Flush FAT metadata and unmount. After this the card is `Empty`.
    pub async fn unmount(&mut self) -> Result<(), SDCardError> {
        let state = core::mem::replace(&mut self.card_state, CardState::Empty);

        match state {
            CardState::Mounted(fs) => {
                warn!("unmounting the sdcard");
                fs.unmount().await.map_err(|_| SDCardError::Io)?;
                self.card_state = CardState::Empty;
                Ok(())
            }

            CardState::Raw(sd) => {
                self.card_state = CardState::Raw(sd);
                Ok(())
            }
            CardState::Empty => Ok(()),
        }
    }

    /// True when the filesystem is mounted
    pub fn is_mounted(&self) -> bool {
        matches!(self.card_state, CardState::Mounted(_))
    }

    /// Create (or truncate) a file and write `data` to it
    pub async fn write_file(&self, name: &str, data: &[u8]) -> Result<(), SDCardError> {
        let CardState::Mounted(fs) = &self.card_state else {
            return Err(SDCardError::NotMounted);
        };
        let root = fs.root_dir();
        let mut file = root.create_file(name).await.map_err(|_| SDCardError::Io)?;
        file.write_all(data).await.map_err(|_| SDCardError::Io)?;
        file.flush().await.map_err(|_| SDCardError::Io)?;
        Ok(())
    }

    /// Append `data` to a file, and create it if it doesn't exist
    pub async fn append(&self, name: &str, data: &[u8]) -> Result<(), SDCardError> {
        let CardState::Mounted(fs) = &self.card_state else {
            return Err(SDCardError::NotMounted);
        };
        let root = fs.root_dir();
        let mut file = root.create_file(name).await.map_err(|_| SDCardError::Io)?;
        file.seek(SeekFrom::End(0)).await.map_err(|_| SDCardError::Io)?;
        file.write_all(data).await.map_err(|_| SDCardError::Io)?;
        file.flush().await.map_err(|_| SDCardError::Io)?;
        Ok(())
    }

    /// Read up to `out.len()` bytes from a file, returning how many were read
    pub async fn read_file(&self, name: &str, out: &mut [u8]) -> Result<usize, SDCardError> {
        let CardState::Mounted(fs) = &self.card_state else {
            return Err(SDCardError::NotMounted);
        };
        let root = fs.root_dir();
        let mut file = root.open_file(name).await.map_err(|_| SDCardError::Io)?;
        file.read(out).await.map_err(|_| SDCardError::Io)
    }

    /// Open a file once and run `f` with it, then flush. Efficient path for
    /// high-rate logging: the file is opened a single time and the closure can
    /// perform many writes before it's flushed and closed.
    ///
    /// The file is created if it doesn't exist (not truncated). Seek to the end
    /// inside the closure if you want to append.
    pub async fn with_file<R>(
        &self,
        name: &str,
        f: impl AsyncFnOnce(&mut SDFile<'_, SPI>) -> Result<R, SDCardError>,
    ) -> Result<R, SDCardError> {
        let CardState::Mounted(fs) = &self.card_state else {
            return Err(SDCardError::NotMounted);
        };
        let root = fs.root_dir();
        let mut file = root.create_file(name).await.map_err(|_| SDCardError::Io)?;
        let result = f(&mut file).await?;
        file.flush().await.map_err(|_| SDCardError::Io)?;
        Ok(result)
    }
}

#[allow(clippy::large_enum_variant)]
enum CardState<SPI: SpiDevice> {
    Empty,
    Raw(SDDev<SPI>),
    Mounted(SDFs<SPI>),
}
