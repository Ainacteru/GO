use atsamd_hal::pwm::{self, Pwm0, Pwm2};
use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::info;
use core::marker::PhantomData;

pub trait ServoPin {
    type Pwm;
    fn channel() -> pwm::Channel;
}

impl ServoPin for crate::Servo1Pwm {
    type Pwm = Pwm2;
    fn channel() -> pwm::Channel { pwm::Channel::_0 }
}
impl ServoPin for crate::Servo2Pwm {
    type Pwm = Pwm0;
    fn channel() -> pwm::Channel { pwm::Channel::_2 }
}
impl ServoPin for crate::Servo3Pwm {
    type Pwm = Pwm0;
    fn channel() -> pwm::Channel { pwm::Channel::_3 }
}



pub struct Servo<P: ServoPin> {
    _marker: PhantomData<P>,
}

impl<P> Servo<P>
where
    P: ServoPin,
    P::Pwm: _embedded_hal_Pwm<Channel = pwm::Channel, Duty = u32>,
{
    pub fn new(_pin: P) -> Self {
        Self { _marker:PhantomData }
    }

    pub fn set_pos(&self, pwm: &mut P::Pwm, angle: u32) {

        const MAX_ANGLE: i32 = 300;
        if angle > 300 {
            panic!("set_pos is {} instead of the max of {}", angle, MAX_ANGLE)
        }

        //let angle = angle.min(300); // safety clamp

        let pulse_width = 1000 + (angle * 1000) / 300;

        let max = pwm.get_max_duty();
        let duty = pulse_width * max / 20000;

        pwm.set_duty(P::channel(), duty);
        info!("angle {}, pulse: {}, duty: {},", angle, pulse_width, duty);
    }
}