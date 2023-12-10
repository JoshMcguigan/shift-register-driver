//! Serial-in parallel-out shift register

use core::cell::RefCell;

use crate::hal::digital::v2::OutputPin;

/// internal use
pub trait ShiftRegisterInternal {
    /// internal error
    type Error;
    /// updates shift register
    fn update(&self, index: usize, command: bool) -> Result<(), Self::Error>;
}

/// Output pin of the shift register
pub struct ShiftRegisterPin<'a, T> {
    shift_register: &'a T,
    index: usize,
}

impl<'a, T: ShiftRegisterInternal> ShiftRegisterPin<'a, T> {
    fn new(shift_register: &'a T, index: usize) -> Self {
        ShiftRegisterPin {
            shift_register,
            index,
        }
    }
}

impl<T: ShiftRegisterInternal> OutputPin for ShiftRegisterPin<'_, T> {
    type Error = <T as ShiftRegisterInternal>::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.shift_register.update(self.index, false)?;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.shift_register.update(self.index, true)?;
        Ok(())
    }
}

macro_rules! ShiftRegisterBuilder {
    ($name: ident, $size: expr) => {
        /// Serial-in parallel-out shift register
        pub struct $name<Pin1, Pin2, Pin3>
        where
            Pin1: OutputPin,
            Pin2: OutputPin,
            Pin3: OutputPin,
        {
            clock: RefCell<Pin1>,
            latch: RefCell<Pin2>,
            data: RefCell<Pin3>,
            output_state: RefCell<[bool; $size]>,
        }

        impl<Pin1, Pin2, Pin3> ShiftRegisterInternal for $name<Pin1, Pin2, Pin3>
        where
            Pin1: OutputPin,
            Pin2: OutputPin,
            Pin3: OutputPin,
        {
            type Error = SRError<
                <Pin1 as OutputPin>::Error,
                <Pin2 as OutputPin>::Error,
                <Pin3 as OutputPin>::Error,
            >;
            /// Sets the value of the shift register output at `index` to value `command`
            fn update(&self, index: usize, command: bool) -> Result<(), Self::Error> {
                self.output_state.borrow_mut()[index] = command;
                let output_state = self.output_state.borrow();
                self.latch
                    .borrow_mut()
                    .set_low()
                    .map_err(|e| SRError::LatchPinError(e))?;

                for i in 1..=output_state.len() {
                    if output_state[output_state.len() - i] {
                        self.data
                            .borrow_mut()
                            .set_high()
                            .map_err(|e| SRError::DataPinError(e))?;
                    } else {
                        self.data
                            .borrow_mut()
                            .set_low()
                            .map_err(|e| SRError::DataPinError(e))?;
                    }
                    self.clock
                        .borrow_mut()
                        .set_high()
                        .map_err(|e| SRError::ClockPinError(e))?;
                    self.clock
                        .borrow_mut()
                        .set_low()
                        .map_err(|e| SRError::ClockPinError(e))?;
                }

                self.latch
                    .borrow_mut()
                    .set_high()
                    .map_err(|e| SRError::LatchPinError(e))?;
                Ok(())
            }
        }

        impl<Pin1, Pin2, Pin3> $name<Pin1, Pin2, Pin3>
        where
            Pin1: OutputPin,
            Pin2: OutputPin,
            Pin3: OutputPin,
        {
            /// Creates a new SIPO shift register from clock, latch, and data output pins
            pub fn new(clock: Pin1, latch: Pin2, data: Pin3) -> Self {
                $name {
                    clock: RefCell::new(clock),
                    latch: RefCell::new(latch),
                    data: RefCell::new(data),
                    output_state: RefCell::new([false; $size]),
                }
            }

            /// Get embedded-hal output pins to control the shift register outputs
            pub fn decompose(&self) -> [ShiftRegisterPin<'_, Self>; $size] {
                let mut result = core::array::from_fn(|_| None);

                for (index, elem) in result.iter_mut().enumerate() {
                    elem.replace(ShiftRegisterPin::<'_, Self>::new(self, index));
                }

                result.map(|v| v.unwrap())
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
        }
    };
}

ShiftRegisterBuilder!(ShiftRegister8, 8);
ShiftRegisterBuilder!(ShiftRegister16, 16);
ShiftRegisterBuilder!(ShiftRegister24, 24);
ShiftRegisterBuilder!(ShiftRegister32, 32);
ShiftRegisterBuilder!(ShiftRegister40, 40);
ShiftRegisterBuilder!(ShiftRegister48, 48);
ShiftRegisterBuilder!(ShiftRegister56, 56);
ShiftRegisterBuilder!(ShiftRegister64, 64);
ShiftRegisterBuilder!(ShiftRegister72, 72);
ShiftRegisterBuilder!(ShiftRegister80, 80);
ShiftRegisterBuilder!(ShiftRegister88, 88);
ShiftRegisterBuilder!(ShiftRegister96, 96);
ShiftRegisterBuilder!(ShiftRegister104, 104);
ShiftRegisterBuilder!(ShiftRegister112, 112);
ShiftRegisterBuilder!(ShiftRegister120, 120);
ShiftRegisterBuilder!(ShiftRegister128, 128);

/// 8 output serial-in parallel-out shift register
pub type ShiftRegister<Pin1, Pin2, Pin3> = ShiftRegister8<Pin1, Pin2, Pin3>;

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
