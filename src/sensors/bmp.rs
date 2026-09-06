use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use defmt::{debug, error, info};
use uom::si::{f32::{Length, Pressure, ThermodynamicTemperature}, length, pressure::{self, pascal}, thermodynamic_temperature};

use crate::sensors::error::BarometerError;

const ADDRESS: u8 = 0x77;

pub struct Bmp<B, D> 
    where 
        B: I2c<SevenBitAddress>,
        D: DelayNs, 
{
    i2c: B,
    delay: D,
    cal_coefs: CompensationCoefficients,
    ref_pressure: Option<Pressure>,
}

impl<B: I2c, D: DelayNs> Bmp<B, D> {
    pub async fn new(i2c: B, delay: D) -> Result<Self, BarometerError> {
        let mut i2c = i2c;

        let mut addr_buf = [0u8; 1];
        i2c.write_read(ADDRESS, &[0x00], &mut addr_buf).await.map_err(|_| BarometerError::I2C)?;

        if addr_buf[0] == 0x60 {
            debug!("addr: {:#02X}", &addr_buf[0]);
        } else {
            error!("Barometer address not matching 0x60, found: {:#02X}", &addr_buf[0])
        }

        let cal = CompensationCoefficients::init_coefficients(&mut i2c).await?;
        let mut baro = Self {
            i2c,
            delay,
            cal_coefs: cal,
            ref_pressure: None,
        };

        baro.write(0x7E, 0xB6).await.map_err(|_| BarometerError::I2C)?;
        baro.delay.delay_ms(10).await;

        // configure osr
        baro.write(0x1C, 0b00_001_011).await.map_err(|_| BarometerError::I2C)?;

        // configure odr
        baro.write(0x1D, 0x03).await.map_err(|_| BarometerError::I2C)?;

        // configure iir
        baro.write(0x1F, 0b0000_0110).await.map_err(|_| BarometerError::I2C)?;

        // configure pwr
        baro.write(0x1B, 0b00_11_00_11).await.map_err(|_| BarometerError::I2C)?;

        baro.delay.delay_ms(50).await;

        let pwr = baro.read(0x1B).await?;    // should read back 0x33
        let err = baro.read(0x02).await?;    // ERR_REG: bit2 = conf_err  <- the one to watch
        let sts = baro.read(0x03).await?;    // STATUS: bit5 drdy_press, bit6 drdy_temp
        debug!("pwr {:#04x} err {:#04x} status {:#04x}", pwr, err, sts);

        Ok(baro)
    }

    async fn read(&mut self, addr: u8) -> Result<u8, BarometerError>{
        let mut buf = [0u8; 1];

        self.i2c.write_read(ADDRESS, &[addr], &mut buf).await.map_err(|_| BarometerError::I2C)?;

        Ok(buf[0])
    }

    async fn write(&mut self, addr: u8, data: u8) -> Result<(), BarometerError>{

        self.i2c.write(ADDRESS, &[addr, data]).await.map_err(|_| BarometerError::I2C)?;

        Ok(())
    }
}

/// actual thing
impl<B: I2c, D: DelayNs> Bmp<B, D> {

    pub async fn read_measurement(&mut self) -> Result<(Pressure, ThermodynamicTemperature), BarometerError> {
        let mut raw = [0u8; 6];
        self.i2c.write_read(ADDRESS, &[0x04], &mut raw).await.map_err(|_| BarometerError::I2C)?;
    
        let uncomp_pres = ((raw[2] as u32) << 16) | ((raw[1] as u32) << 8) | (raw[0] as u32);
        let uncomp_temp = ((raw[5] as u32) << 16) | ((raw[4] as u32) << 8) | (raw[3] as u32);
    
        let cal = &mut self.cal_coefs;
        let pd1 = uncomp_temp as f32 - cal.PAR_T1;
        let pd2 = pd1 * cal.PAR_T2;
        cal.t_lin = pd2 + (pd1 * pd1) * cal.PAR_T3;

        let partial_data1 = cal.PAR_P6 * cal.t_lin;
        let partial_data2 = cal.PAR_P7 * (cal.t_lin * cal.t_lin);
        let partial_data3 = cal.PAR_P8 * (cal.t_lin * cal.t_lin * cal.t_lin);
        let partial_out1 = cal.PAR_P5 + partial_data1 + partial_data2 + partial_data3;

        let partial_data1 = cal.PAR_P2 * cal.t_lin;
        let partial_data2 = cal.PAR_P3 * (cal.t_lin * cal.t_lin);
        let partial_data3 = cal.PAR_P4 * (cal.t_lin * cal.t_lin * cal.t_lin);
        let partial_out2 = uncomp_pres as f32 * (cal.PAR_P1 + partial_data1 + partial_data2 + partial_data3);

        let partial_data1 = uncomp_pres as f32 * uncomp_pres as f32 ;
        let partial_data2 = cal.PAR_P9 + cal.PAR_P10 * cal.t_lin;
        let partial_data3 = partial_data1 * partial_data2;
        let partial_data4 = partial_data3 + (uncomp_pres as f32 * uncomp_pres as f32 * uncomp_pres as f32 ) * cal.PAR_P11;
        let comp_pres = partial_out1 + partial_out2 + partial_data4;

        // debug!("raw_p {} raw_t {} comp_p {} t_lin {}", uncomp_pres, uncomp_temp, comp_pres, cal.t_lin);

        Ok((
            Pressure::new::<pressure::pascal>(comp_pres), 
            ThermodynamicTemperature::new::<thermodynamic_temperature::degree_celsius>(cal.t_lin),
        ))
    }

    pub async fn get_altitude(&mut self) -> Result<Length, BarometerError> {
        const R_SPECIFIC: f32 = 287.05;
        const G: f32 = 9.80665;

        let (pressure, temperature) = self.read_measurement().await?;
        let p = pressure.get::<pascal>();

        let p_ref = self.ref_pressure.get_or_insert(pressure).get::<pascal>();
        let t = temperature.get::<thermodynamic_temperature::kelvin>();

        Ok(Length::new::<length::meter>(
            (R_SPECIFIC * t / G) * libm::logf(p_ref / p),
        ))
    }
}

#[allow(non_snake_case)]
struct CompensationCoefficients {
    t_lin: f32,
    PAR_T1: f32,
    PAR_T2: f32,
    PAR_T3: f32,
    PAR_P1: f32,
    PAR_P2: f32,
    PAR_P3: f32,
    PAR_P4: f32,
    PAR_P5: f32,
    PAR_P6: f32,
    PAR_P7: f32,
    PAR_P8: f32,
    PAR_P9: f32,
    PAR_P10: f32,
    PAR_P11: f32,
}

// PAR_T1: u16,
// PAR_T2: u16,
// PAR_T3: i8,
// PAR_P1: i16,
// PAR_P2: i16,
// PAR_P3: i8,
// PAR_P4: i8,
// PAR_P5: u16,
// PAR_P6: u16,
// PAR_P7: i8,
// PAR_P8: i8,
// PAR_P9: i16,
// PAR_P10: i8,
// PAR_P11: i8,

impl CompensationCoefficients {
    async fn init_coefficients<B: I2c<SevenBitAddress>>(i2c: &mut B) -> Result<Self, BarometerError> {
        let mut trimming_coeffs = [0u8; 21];
        i2c.write_read(ADDRESS, &[0x31], &mut trimming_coeffs).await.map_err(|_| BarometerError::I2C)?;

        Ok( Self {
            t_lin: 0.0,
            PAR_T1: u16::from_le_bytes(trimming_coeffs[0..2].try_into().map_err(BarometerError::Array)?) as f32 / 0.00390625,
            PAR_T2: u16::from_le_bytes(trimming_coeffs[2..4].try_into().map_err(BarometerError::Array)?) as f32 / 1073741824.0,
            PAR_T3: trimming_coeffs[4] as i8 as f32 / 281474976710656.0,
            PAR_P1: (i16::from_le_bytes(trimming_coeffs[5..7].try_into().map_err(BarometerError::Array)?) as f32 - 16384.0) / 1048576.0,
            PAR_P2: (i16::from_le_bytes(trimming_coeffs[7..9].try_into().map_err(BarometerError::Array)?) as f32 - 16384.0) / 536870912.0,
            PAR_P3: trimming_coeffs[9] as i8 as f32 / 4294967296.0,
            PAR_P4: trimming_coeffs[10] as i8 as f32 / 137438953472.0,
            PAR_P5: u16::from_le_bytes(trimming_coeffs[11..13].try_into().map_err(BarometerError::Array)?) as f32 / 0.125,
            PAR_P6: u16::from_le_bytes(trimming_coeffs[13..15].try_into().map_err(BarometerError::Array)?) as f32 / 64.0,
            PAR_P7: trimming_coeffs[15] as i8 as f32 / 256.0,
            PAR_P8: trimming_coeffs[16] as i8 as f32 / 32768.0,
            PAR_P9: i16::from_le_bytes(trimming_coeffs[17..19].try_into().map_err(BarometerError::Array)?) as f32 / 281474976710656.0,
            PAR_P10: trimming_coeffs[19] as i8 as f32 / 281474976710656.0,
            // PAR_P11: trimming_coeffs[20] as i8 as f32 / 3.689348814741910e19,
            PAR_P11: trimming_coeffs[20] as i8 as f32 / 3.689_349e19,
        })
    }
}

