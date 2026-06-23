use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{defs::Reg, defs::*, Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub(crate) async fn read<const N: usize, T: TryFrom<[u8; N]>>(&mut self, reg: Reg) -> Result<T, Error<E>> {
    let mut b = [0u8; N];
    self.read_bytes(reg, &mut b).await?;
    TryFrom::try_from(b).map_err(|_| Error::Data)
  }

  pub(crate) async fn read_u8(&mut self, reg: Reg) -> Result<u8, Error<E>> {
    let mut b = [0u8; 1];
    self.read_bytes(reg, &mut b).await?;
    Ok(b[0])
  }

  pub(crate) async fn read_u16(&mut self, reg: Reg) -> Result<u16, Error<E>> {
    let mut b = [0u8; 2];
    self.read_bytes(reg, &mut b).await?;
    Ok(u16::from_le_bytes(b))
  }

  pub(crate) async fn read_bytes(&mut self, reg: Reg, buf: &mut [u8]) -> Result<(), Error<E>> {
    // The device returns two dummy bytes before the actual register data.
    // Read into a temporary buffer and skip the first two bytes.
    debug_assert!(buf.len() <= 30, "read_bytes buffer too large");

    let mut tmp = [0u8; 32];
    let read_len = buf.len() + 2; // two dummy bytes in front

    self
      .i2c
      .write_read(ADDR_I2C_PRIM, &[reg as u8], &mut tmp[..read_len])
      .await
      .map_err(Error::I2c)?;

    buf.copy_from_slice(&tmp[2..read_len]);
    Ok(())
  }

  pub(crate) async fn write<const N: usize, T: TryInto<[u8; N]>>(&mut self, reg: Reg, v: T) -> Result<(), Error<E>> {
    let b = v.try_into().map_err(|_| Error::Data)?;
    self.write_bytes(reg, &b).await
  }

  pub(crate) async fn write_u16(&mut self, reg: Reg, value: u16) -> Result<(), Error<E>> {
    // Device registers use little‑endian ordering (LSB first).
    self.write_bytes(reg, &value.to_le_bytes()).await
  }

  pub(crate) async fn write_bytes(&mut self, reg: Reg, data: &[u8]) -> Result<(), Error<E>> {
    debug_assert!(data.len() <= 31, "write_bytes buffer too small");
    let mut buf = [0u8; 32];
    let len = 1 + data.len();
    buf[0] = reg as u8;
    buf[1..len].copy_from_slice(data);
    self.i2c.write(ADDR_I2C_PRIM, &buf[..len]).await.map_err(Error::I2c)?;
    self.delay.delay_us(20).await;
    Ok(())
  }
}
