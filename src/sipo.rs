//! Serial-in parallel-out shift register

use core::cell::RefCell;

use hal::digital::{self, ErrorType};

use crate::hal::digital::OutputPin;

type SRErr<Pin1, Pin2, Pin3> = SRError<<Pin1 as ErrorType>::Error, <Pin2 as ErrorType>::Error, <Pin3 as ErrorType>::Error>;
/// Output pin of the shift register
pub struct ShiftRegisterPin<'a, Pin1, Pin2, Pin3, const N: usize>
where
    Pin1: OutputPin,
    Pin2: OutputPin,
    Pin3: OutputPin,
{
    shift_register: &'a ShiftRegister<Pin1, Pin2, Pin3, N>,
    index: usize,
}

impl<'a, Pin1, Pin2, Pin3, const N: usize> ShiftRegisterPin<'a, Pin1, Pin2, Pin3, N>
where
    Pin1: OutputPin,
    Pin2: OutputPin,
    Pin3: OutputPin,
{
    fn new(shift_register: &'a ShiftRegister<Pin1, Pin2, Pin3, N>, index: usize) -> Self {
        ShiftRegisterPin {
            shift_register,
            index,
        }
    }
}

impl<Pin1, Pin2, Pin3, const N: usize> ErrorType for ShiftRegisterPin<'_, Pin1, Pin2, Pin3, N> 
    where
        Pin1: OutputPin + core::fmt::Debug,
        Pin2: OutputPin + core::fmt::Debug,
        Pin3: OutputPin + core::fmt::Debug,
{
    type Error = SRErr<Pin1, Pin2, Pin3>;
}
impl<Pin1, Pin2, Pin3, const N: usize> OutputPin for ShiftRegisterPin<'_, Pin1, Pin2, Pin3, N>
where
    Pin1: OutputPin + core::fmt::Debug,
    Pin2: OutputPin + core::fmt::Debug,
    Pin3: OutputPin + core::fmt::Debug,
{

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.shift_register.update(self.index, false)?;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.shift_register.update(self.index, true)?;
        Ok(())
    }
}

/// Serial-in parallel-out shift register
pub struct ShiftRegister<Pin1, Pin2, Pin3, const N: usize>
where
    Pin1: OutputPin,
    Pin2: OutputPin,
    Pin3: OutputPin,
{
    clock: RefCell<Pin1>,
    latch: RefCell<Pin2>,
    data: RefCell<Pin3>,
    output_state: RefCell<[bool; N]>,
}

impl<Pin1, Pin2, Pin3, const N: usize> ShiftRegister<Pin1, Pin2, Pin3, N>
where
    Pin1: OutputPin,
    Pin2: OutputPin,
    Pin3: OutputPin,
{
    /// Creates a new SIPO shift register from clock, latch, and data output pins
    pub fn new(clock: Pin1, latch: Pin2, data: Pin3) -> Self {
        ShiftRegister {
            clock: RefCell::new(clock),
            latch: RefCell::new(latch),
            data: RefCell::new(data),
            output_state: RefCell::new([false; N]),
        }
    }

    /// Get embedded-hal output pins to control the shift register outputs
    pub fn decompose(&self) -> [ShiftRegisterPin<'_, Pin1, Pin2, Pin3, N>; N] {
        core::array::from_fn(|i| ShiftRegisterPin::<'_, Pin1, Pin2, Pin3, N>::new(self, i))
    }

    /// Consume the shift register and return the original clock, latch, and data output pins
    pub fn release(self) -> (Pin1, Pin2, Pin3) {
        let Self {
            clock,
            latch,
            data,
            output_state: _,
        } = self;
        (clock.into_inner(), latch.into_inner(), data.into_inner())
    }

    fn update(
        &self,
        index: usize,
        command: bool,
    ) -> Result<
        (),
        SRErr<Pin1, Pin2, Pin3>,
    > {
        self.output_state.borrow_mut()[index] = command;
        let output_state = self.output_state.borrow();
        self.latch
            .borrow_mut()
            .set_low()
            .map_err(SRError::LatchPinError)?;

        for i in 1..=output_state.len() {
            if output_state[output_state.len() - i] {
                self.data
                    .borrow_mut()
                    .set_high()
                    .map_err(SRError::DataPinError)?;
            } else {
                self.data
                    .borrow_mut()
                    .set_low()
                    .map_err(SRError::DataPinError)?;
            }
            self.clock
                .borrow_mut()
                .set_high()
                .map_err(SRError::ClockPinError)?;
            self.clock
                .borrow_mut()
                .set_low()
                .map_err(SRError::ClockPinError)?;
        }

        self.latch
            .borrow_mut()
            .set_high()
            .map_err(SRError::LatchPinError)?;
        Ok(())
    }
}

/// Error type during update
#[derive(Debug)]
pub enum SRError<Pin1Err, Pin2Err, Pin3Err> {
    /// Something wrong with the clock pin.
    ClockPinError(Pin1Err),
    /// Something wrong with the latch pin.
    LatchPinError(Pin2Err),
    /// Something wrong with the data pin.
    DataPinError(Pin3Err),
}

impl<Pin1Err, Pin2Err, Pin3Err> digital::Error for SRError<Pin1Err, Pin2Err, Pin3Err> 
where
    Pin1Err: core::fmt::Debug,
    Pin2Err: core::fmt::Debug,
    Pin3Err: core::fmt::Debug,
{
    fn kind(&self) -> digital::ErrorKind {
        digital::ErrorKind::Other
    }
}