pub mod rgb_leds{
    use core::marker::PhantomData;

use atsamd_hal::pwm::{self, Pwm0};

    pub trait LEDPin {
        fn channel() -> pwm::Channel;
    }

    impl LEDPin for crate::RgbRed {
        fn channel() -> pwm::Channel { pwm::Channel::_2 }
    }
    impl LEDPin for crate::RgbGreen {
        fn channel() -> pwm::Channel { pwm::Channel::_1 }
    }
    impl LEDPin for crate::RgbBlue {
        fn channel() -> pwm::Channel { pwm::Channel::_3 }
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
        
    }
}

