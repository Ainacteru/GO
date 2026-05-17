use atsamd_hal::pwm::{self, Pwm0};
use cortex_m::prelude::_embedded_hal_Pwm;
use core::marker::PhantomData;

use crate::uprintln;

pub trait ServoPin {
    fn channel() -> pwm::Channel;
}

impl ServoPin for crate::Servo1Pwm {
    fn channel() -> pwm::Channel { pwm::Channel::_2 }
}
impl ServoPin for crate::Servo2Pwm {
    fn channel() -> pwm::Channel { pwm::Channel::_1 }
}
impl ServoPin for crate::Servo3Pwm {
    fn channel() -> pwm::Channel { pwm::Channel::_3 }
}



pub struct Servo<P: ServoPin> {
    _marker: PhantomData<P>,
}

impl<P> Servo<P> 
where P: ServoPin
{
    pub fn new(_pin: P) -> Self {
        Self { _marker:PhantomData }
    }

    pub fn set_pos(&self, pwm: &mut Pwm0, angle: u32) {

        const MAX_ANGLE: i32 = 300;
        if angle > 300 {
            panic!("set_pos is {} instead of the max of {}", angle, MAX_ANGLE)
        }

        let angle = angle.min(300); // safety clamp

        let pulse_width = 1000 + (angle * 1000) / 300;

        let max = pwm.get_max_duty();
        let duty = pulse_width * max / 20000;

        pwm.set_duty(P::channel(), duty);
        uprintln!("angle {}, pulse: {}, duty: {},", angle, pulse_width, duty);
    }
}