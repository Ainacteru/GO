use bmp390_rs::{
    Bmp390Result, ResetPolicy, SdoPinState, bus::I2c as BmpI2c, typestate::{Bmp390Builder, Bmp390Mode, NoPin, Normal, PressureAndTemperature}
};
use atsamd_hal::ehal_async::{delay::DelayNs, i2c::I2c};
use defmt::info;
use uom::si::{f32::{Length, Pressure}, pressure::pascal};

type Inner<B, D> = Bmp390Mode<Normal, PressureAndTemperature, BmpI2c<B>, NoPin, D, false>;

pub struct Bmp<B: I2c, D: DelayNs> {
    inner: Inner<B, D>,
    prev_alt: Length,
}

impl<B: I2c, D: DelayNs> Bmp<B, D> {
    pub async fn new(i2c: B, delay: D) -> Bmp390Result<Self, <BmpI2c<B> as bmp390_rs::bus::Bus>::Error> {
        let inner = Bmp390Builder::new()
            .use_i2c(i2c, SdoPinState::High)
            .enable_temperature()
            .enable_pressure()
            .into_normal()
            .build(ResetPolicy::Soft, delay)
            .await?;

        info!("BMP390 id: {:#x}", 0x60_u8);

        Ok(Self {
            inner,
            prev_alt: Length::new::<uom::si::length::meter>(0.0),
        })
    }
    pub fn inner(&mut self) -> &mut Inner<B, D> {
        &mut self.inner
    }
    pub fn elevation_from_pressure(&self, pres: Pressure) -> Length {
        let p = pres.get::<pascal>();
        let meters = 44_330.0 * (1.0 - libm::powf(p / 101_325.0, 0.19026));
        Length::new::<uom::si::length::meter>(meters)
    }
    pub async fn altitude(&mut self) -> Length {
        let mes = self.inner().read_latest_measurement().await.unwrap().into_uom();
        let pres = mes.pressure_pascal();
        let elevation = self.elevation_from_pressure(pres);

        if self.prev_alt.get::<uom::si::length::meter>() == 0.0 {
            self.prev_alt = elevation;
        }

        elevation - self.prev_alt
    }
}
