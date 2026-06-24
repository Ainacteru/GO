use core::marker::PhantomData;

use atsamd_hal::{fugit::RateExtU32, pwm::{Channel, Pwm2}};
use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::{info, warn};

use crate::Buzzer as aaaa;

pub struct Buzzer {
    _buzzer: PhantomData<aaaa>,
    pwm: Pwm2
}

impl Buzzer {
    pub fn new(_buzzer: aaaa, pwm: Pwm2) -> Self {
        Self {
            _buzzer: PhantomData,
            pwm
        }
    }

    pub fn set_volume(&mut self, volume: u32) {
        let mut volume = volume;
        if volume > 100 {
            warn!("Buzzer volume was set to {}, setting it to 100", &volume);
            volume = 100;
        }
        let max = self.pwm.get_max_duty() / 20;
        
        let duty = max * volume / 100;
        info!("{}", duty);
        self.pwm.set_duty(Channel::_1, duty);

    }

    pub fn set_pitch(&mut self, frequency: u32) {
        let mut frequency = frequency;
        if frequency > 2700 {
            warn!("Buzzer volume was set to {}, setting it to 100", &frequency);
            frequency = 2700;
        }

        self.pwm.set_period(frequency.Hz());
    }

    pub fn set_note(&mut self, note: Note) {
        self.pwm.set_period((note.hz() as u32 + 1).Hz());
    }
}

pub enum Note {
    C4,
    Cs4,
    D4,
    Ds4,
    E4,
    F4,
    Fs4,
    G4,
    Gs4,
    A4,
    As4,
    B4,
    C5,
    Cs5,
    D5,
    Ds5,
    E5,
    F5,
    Fs5,
    G5,
    Gs5,
    A5,
    As5,
    B5,
}

impl Note {
    pub const fn hz(&self) -> f32 {
        match self {
            Self::C4 => 261.63,
            Self::Cs4 => 277.18,
            Self::D4 => 293.66,
            Self::Ds4 => 311.13,
            Self::E4 => 329.63,
            Self::F4 => 349.23,
            Self::Fs4 => 369.99,
            Self::G4 => 392.00,
            Self::Gs4 => 415.30,
            Self::A4 => 440.00,
            Self::As4 => 466.16,
            Self::B4 => 493.88,
            Self::C5 => 523.25,
            Self::Cs5 => 554.37,
            Self::D5 => 587.33,
            Self::Ds5 => 622.25,
            Self::E5 => 659.25,
            Self::F5 => 698.46,
            Self::Fs5 => 739.99,
            Self::G5 => 783.99,
            Self::Gs5 => 830.61,
            Self::A5 => 880.00,
            Self::As5 => 932.33,
            Self::B5 => 987.77,
        }
    }
}