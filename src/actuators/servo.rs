use atsamd_hal::gpio::{AlternateF, PA16, Pin, PinId, PinMode};

use crate::Pins;

enum Servos {
    Servo1,
    Servo2,
    Servo3,
}

impl Servos {
    fn get_pin<I,M>(&self, pins: Pins) -> Pin<I,M> 
    where
        I: PinId, M: PinMode
    {
        match self {
            Servos::Servo1 => pins.servo1.into_alternate(AlternateF), 
            Servos::Servo2 => Pin<PA18, AlternateF>,
            Servos::Servo3 => Pin<PA19, AlternateF>,
        }
    }
}

struct Servo {
    servo: Servos,
}

impl Servo{
    pub fn new(servo: Servos, ) {
        
    }
}

