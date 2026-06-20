use core::cell::RefCell;
use core::task::Waker;

use critical_section::Mutex;
use embassy_time_driver::Driver;
use portable_atomic::{AtomicU64, Ordering};

use crate::pac::{interrupt, Tc3};

const ALARM_COUNT: usize = 4;

struct AlarmState {
    at: u64,
    waker: Option<Waker>,
}

impl AlarmState {
    const fn new() -> Self {
        Self { at: u64::MAX, waker: None }
    }
}

pub struct Samd21Driver {
    ticks: AtomicU64,
    alarms: Mutex<RefCell<[AlarmState; ALARM_COUNT]>>,
}

unsafe impl Sync for Samd21Driver {}

impl Samd21Driver {
    const fn new() -> Self {
        Self {
            ticks: AtomicU64::new(0),
            alarms: Mutex::new(RefCell::new([
                AlarmState::new(),
                AlarmState::new(),
                AlarmState::new(),
                AlarmState::new(),
            ])),
        }
    }
}

impl Driver for Samd21Driver {
    fn now(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut alarms = self.alarms.borrow(cs).borrow_mut();
            let slot = alarms
                .iter()
                .position(|a| match &a.waker {
                    Some(w) => w.will_wake(waker),
                    None => false,
                })
                .or_else(|| alarms.iter().position(|a| a.waker.is_none()))
                .unwrap_or(0);
            alarms[slot].at = at;
            alarms[slot].waker = Some(waker.clone());
        });
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: Samd21Driver = Samd21Driver::new());

/// Initialize TC3 as the embassy time driver.
/// Call this early in main, after clocks are configured.
pub fn init(tc3: Tc3, pm: &mut crate::pac::Pm, clocks: &mut crate::hal::clock::GenericClockController) {
    let gclk0 = clocks.gclk0();
    let _clock = clocks.tcc2_tc3(&gclk0).unwrap();

    // Enable TC3 in PM
    pm.apbcmask().modify(|_, w| w.tc3_().set_bit());

    let tc = tc3.count16();

    // Software reset
    tc.ctrla().write(|w| w.swrst().set_bit());
    while tc.status().read().syncbusy().bit_is_set() {}
    // Extra wait for reset to complete
    cortex_m::asm::nop();

    // Configure: 16-bit, MFRQ (reset on CC0 match), prescaler DIV64
    // 48MHz / 64 = 750kHz. CC[0] = 749 → period = 1ms
    tc.ctrla().write(|w| {
        w.mode().count16();
        w.wavegen().mfrq();
        w.prescaler().div64();
        w.enable().set_bit()
    });

    // Wait for sync
    while tc.status().read().syncbusy().bit_is_set() {}

    tc.cc(0).write(|w| unsafe { w.cc().bits(749) });
    while tc.status().read().syncbusy().bit_is_set() {}

    // Enable MC0 interrupt (fires on CC0 match in MFRQ mode)
    tc.intenset().write(|w| w.mc0().set_bit());

    // Enable TC3 interrupt in NVIC
    unsafe { cortex_m::peripheral::NVIC::unmask(crate::pac::Interrupt::TC3) };
}

#[interrupt]
fn TC3() {
    let tc = unsafe { &*Tc3::ptr() }.count16();
    tc.intflag().write(|w| w.mc0().set_bit());

    let now = DRIVER.ticks.fetch_add(1, Ordering::Relaxed) + 1;

    critical_section::with(|cs| {
        let mut alarms = DRIVER.alarms.borrow(cs).borrow_mut();
        for alarm in alarms.iter_mut() {
            if alarm.at <= now {
                alarm.at = u64::MAX;
                if let Some(w) = alarm.waker.take() {
                    w.wake();
                }
            }
        }
    });
}

defmt::timestamp!("{=u64}.{=u64:03}s", embassy_time::Instant::now().as_millis() / 1000, embassy_time::Instant::now().as_millis() % 1000);