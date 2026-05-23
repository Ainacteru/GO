pub mod rgb_leds{
    use core::marker::PhantomData;

    use atsamd_hal::pwm::{self, Pwm0};
    use cortex_m::prelude::_embedded_hal_Pwm;

    use crate::uprintln;

    pub trait LEDPin {
        fn channel() -> pwm::Channel;
    }

    impl LEDPin for crate::RgbRedPwm {
        fn channel() -> pwm::Channel { pwm::Channel::_3 }
    }
    impl LEDPin for crate::RgbGreenPwm {
        fn channel() -> pwm::Channel { pwm::Channel::_1 }
    }
    impl LEDPin for crate::RgbBluePwm {
        fn channel() -> pwm::Channel { pwm::Channel::_0 }
    }

    pub struct RgbLed<P: LEDPin> {
        _marker: PhantomData<P>,
    }

    impl<P> RgbLed<P> 
    where P: LEDPin
    {
        pub fn new(_pin: P) -> Self {
            Self { _marker:PhantomData }
        }

        pub fn set_brightness(&self, pwm: &mut Pwm0, brightness: u32) {
            let max = pwm.get_max_duty();

            let b = max * brightness / 100;

            pwm.set_duty(P::channel(), b);
            //uprintln!("brightness: {}, actual: {}", brightness, b);
        }
    }
}

