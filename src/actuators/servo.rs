use core::{ptr};

use crate::Pins;
use atsamd_hal::{prelude, pwm::Channel};
use cortex_m::prelude::_embedded_hal_Pwm;

#[derive(Copy, Clone)]
pub enum Servos {
    Servo1,
    Servo2,
    Servo3,
}

impl Servos {
    pub fn activate(self, pins: crate::Pins) {
        match self {
            Servos::Servo1 => {let _: crate::pins::Servo1Pwm = pins.servo1.into();},
            Servos::Servo2 => {let _: crate::pins::Servo2Pwm = pins.servo2.into();},
            Servos::Servo3 => {let _: crate::pins::Servo3Pwm = pins.servo3.into();},
        }
    }
}

pub struct Servo {
    servo: Servos,
    pwm: atsamd_hal::pwm::Pwm0,
}

impl Servo{
    pub fn new(servo: Servos, pwm: atsamd_hal::pwm::Pwm0, pins: crate::Pins) -> Servo {
        servo.activate(pins);
        Servo {
            servo,
            pwm,
        } 
    }

    pub fn set_position(&mut self, position: u32) {
        match self.servo {
            Servos::Servo1 => {
                let max_duty = self.pwm.get_max_duty();
                self.pwm.set_duty(Channel::_2, max_duty/position);
            },
            Servos::Servo2 => todo!(),
            Servos::Servo3 => todo!(),
        }
    }
}
            
            