use atsamd_hal::{ehal::i2c::I2c as I2cTrait, pac::rtc::mode0::comp,};
use crate::peripherals::i2c::I2c;

const ADDRESS: u8 = 0x77;

pub struct NVMregs {
    t_lin: f32,

    temperature: TempComp,
    pressure: PressureComp,
}

pub struct TempComp {
    par_t1: f32,
    par_t2: f32,
    par_t3: f32,
}

pub struct PressureComp {
    par_p1: f32,
    par_p2: f32,
    par_p3: f32,
    par_p4: f32,
    par_p5: f32,
    par_p6: f32,
    par_p7: f32,
    par_p8: f32,
    par_p9: f32,
    par_p10: f32,
    par_p11: f32,
}

pub struct Readings {
    altitude: f32,
    started: bool,
}

pub struct Bmp {
    i2c: I2c,
    calibration: NVMregs,
    readings: Readings,
}

impl Bmp {
    pub fn new(mut i2c: I2c) -> Self {
        let mut calib = [0u8; 21];
        i2c.inner().write_read(ADDRESS, &[0x31], &mut calib).unwrap();
        i2c.inner().write(ADDRESS, &[0x1B, 0x33]).unwrap(); 

        Self {
            i2c,
            calibration: NVMregs::new(calib),
            readings: Readings { altitude: 0.0, started: false }
        }
    }

    // in celcius 
    pub fn read_temperature(&mut self) -> f32 {
        let mut buf = [0u8; 3];
        self.i2c.inner().write_read(ADDRESS, &[0x07], &mut buf).unwrap();
        let raw = (buf[2] as u32) << 16 | (buf[1] as u32) << 8 | buf[0] as u32;

        let partial1 = raw as f32 - self.calibration.temperature.par_t1;
        let partial2 = partial1 * self.calibration.temperature.par_t2;
        self.calibration.t_lin = partial2 + (partial1 * partial1) * self.calibration.temperature.par_t3;
        self.calibration.t_lin

        // partial_data1 = (float)(uncomp_temp – calib_data->par_t1);
        // partial_data2 = (float)(partial_data1 * calib_data->par_t2);
        
        // calib_data->t_lin = partial_data2 + (partial_data1 * partial_data1) * calib_data->par_t3;

        // return calib_data->t_lin;
    }   

    pub fn read_pressure(&mut self) -> f32 {
        let mut buf = [0u8; 3];
        self.i2c.inner().write_read(ADDRESS, &[0x04], &mut buf);

        let raw = (buf[2] as u32) << 16 | (buf[1] as u32) << 8 | buf[0] as u32;
        let t_lin = self.calibration.t_lin;

        let partial_data1 = self.calibration.pressure.par_p6 * t_lin;
        let partial_data2 = self.calibration.pressure.par_p7 * (t_lin * t_lin);
        let partial_data3 = self.calibration.pressure.par_p8 * (t_lin * t_lin* t_lin);
        let partial_out1 = self.calibration.pressure.par_p5 + partial_data1 + partial_data2 + partial_data3;

        let partial_data1 = self.calibration.pressure.par_p2 * t_lin;
        let partial_data2 = self.calibration.pressure.par_p3 * (t_lin * t_lin);
        let partial_data3 = self.calibration.pressure.par_p4 * (t_lin * t_lin * t_lin);
        let partial_out2 = raw as f32 * (self.calibration.pressure.par_p1 + partial_data1 + partial_data2 + partial_data3);
        
        let partial_data1 = raw as f32 * raw as f32;
        let partial_data2 = self.calibration.pressure.par_p9 + self.calibration.pressure.par_p10  * t_lin;
        let partial_data3 = partial_data1 * partial_data2;
        let partial_data4 = partial_data3 + (raw as f32 * raw as f32 * raw as f32 * self.calibration.pressure.par_p11);
        
        partial_out1 + partial_out2 + partial_data4 // comp_press


        // partial_data1 = calib_data->par_p6 * calib_data->t_lin;
        // partial_data2 = calib_data->par_p7 * (calib_data->t_lin * calib_data->t_lin);
        // partial_data3 = calib_data->par_p8 * (calib_data->t_lin * calib_data->t_lin * calib_data->t_lin);
        // partial_out1 = calib_data->par_p5 + partial_data1 + partial_data2 + partial_data3;

        // partial_data1 = calib_data->par_p2 * calib_data->t_lin;
        // partial_data2 = calib_data->par_p3 * (calib_data->t_lin * calib_data->t_lin);
        // partial_data3 = calib_data->par_p4 * (calib_data->t_lin * calib_data->t_lin * calib_data->t_lin);
        // partial_out2 = (float)uncomp_press * (calib_data->par_p1 + partial_data1 + partial_data2 + partial_data3);

        // partial_data1 = (float)uncomp_press * (float)uncomp_press;
        // partial_data2 = calib_data->par_p9 + calib_data->par_p10 * calib_data->t_lin;
        // partial_data3 = partial_data1 * partial_data2;
        // partial_data4 = partial_data3 + ((float)uncomp_press * (float)uncomp_press * (float)uncomp_press) * calib_data->par_p11;
        // comp_press = partial_out1 + partial_out2 + partial_data4;

        // return comp_press;
    }

    pub fn get_altitude(& mut self) -> f32 {
        let now = 44330.0 * (1.0 - libm::powf(self.read_pressure() / 101325.0 ,0.1903) );

        if !self.readings.started{
            self.readings.altitude = now;
            self.readings.started = true;
        }


        now - self.readings.altitude
    }
}

impl NVMregs {
    pub fn new(calibration: [u8; 21]) -> Self {
        Self {
            t_lin: 0.0,
            temperature: TempComp::new(&calibration),
            pressure: PressureComp::new(&calibration),
        }   
    }
}

impl TempComp {
    pub fn new(cal: &[u8; 21]) -> Self {
        Self { 
            par_t1: u16::from_le_bytes([cal[0], cal[1]]) as f32 / 0.00390625,
            par_t2: u16::from_le_bytes([cal[2], cal[3]]) as f32 / 1073741824.0, 
            par_t3: cal[4] as i8 as f32 / 281474976710656.0 
        }
    }
}

impl PressureComp {
    pub fn new(cal: &[u8; 21]) -> Self {
        Self {
            par_p1:  (i16::from_le_bytes([cal[5],  cal[6]])  as f32 - 16384.0) / 1048576.0,
            par_p2:  (i16::from_le_bytes([cal[7],  cal[8]])  as f32 - 16384.0) / 536870912.0,
            par_p3:  cal[9] as i8 as f32 / 4294967296.0,
            par_p4:  cal[10] as i8 as f32 / 137438953472.0,
            par_p5:  u16::from_le_bytes([cal[11], cal[12]]) as f32 / 0.125,
            par_p6:  u16::from_le_bytes([cal[13], cal[14]]) as f32 / 64.0,
            par_p7:  cal[15] as i8 as f32 / 256.0,
            par_p8:  cal[16] as i8 as f32 / 32768.0,
            par_p9:  i16::from_le_bytes([cal[17], cal[18]]) as f32 / 281474976710656.0,
            par_p10: cal[19] as i8 as f32 / 281474976710656.0,
            par_p11: cal[20] as i8 as f32 / 36893488147419103232.0,
        }
    }
}