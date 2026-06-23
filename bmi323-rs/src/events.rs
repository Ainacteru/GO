use embedded_hal_async::{delay::DelayNs, digital, i2c::*};

use super::{interrupt::*, Bmi323, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
  Tap(u8),
  NoMotion,
  AnyMotion,
  Flat,
  Orientation { pl: OrientationPl, face: Face },
  StepDetector,
  StepCounter,
  SigMotion,
  Tilt,
  TempDataReady,
  GyrDataReady,
  AccelDataReady,
  FifoWatermark,
  FifoFull,
  ErrStatus,
}

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
  W: digital::Wait,
{
  pub async fn wait_event(&mut self) -> Result<Event, Error<E>> {
    loop {
      if let Some(evt) = self.dequeue.pop_front() {
        return Ok(evt);
      }

      self.int_pin.wait_for_any_edge().await.map_err(|_| Error::Data)?;
      self.push_int1_events().await?;
      if let Some(evt) = self.dequeue.pop_front() {
        return Ok(evt);
      }
    }
  }

  /// Read and decode INT1 (feature) status and append events to the provided queue.
  async fn push_int1_events(&mut self) -> Result<(), Error<E>> {
    let mut tap_event: Option<Event> = None;
    let mut orient_event: Option<Event> = None;

    let st = self.get_int1_status().await?;
    if st.no_motion {
      self.push_event(Event::NoMotion);
    }
    if st.any_motion {
      self.push_event(Event::AnyMotion);
    }
    if st.flat {
      self.push_event(Event::Flat);
    }
    if st.step_detector {
      self.push_event(Event::StepDetector);
    }
    if st.step_counter {
      self.push_event(Event::StepCounter);
    }
    if st.sig_motion {
      self.push_event(Event::SigMotion);
    }
    if st.tilt {
      self.push_event(Event::Tilt);
    }

    if st.tap {
      let ext = self.get_feature_event_ext().await?;
      if ext.s_tap {
        tap_event = Some(Event::Tap(1));
      } else if ext.d_tap {
        tap_event = Some(Event::Tap(2));
      } else if ext.t_tap {
        tap_event = Some(Event::Tap(3));
      }
    }

    if st.orientation {
      let ext = self.get_feature_event_ext().await?;
      orient_event = Some(Event::Orientation { pl: ext.pl, face: ext.face })
    }

    if let Some(e) = tap_event {
      self.push_event(e);
    }

    if let Some(e) = orient_event {
      self.push_event(e);
    }
    Ok(())
  }

  #[inline]
  fn push_event(&mut self, e: Event) {
    if self.dequeue.is_full() {
      let _ = self.dequeue.pop_front();
    }
    let _ = self.dequeue.push_back(e);
  }
}
