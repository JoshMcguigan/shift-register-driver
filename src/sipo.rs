//! Serial-in parallel-out shift register

use core::cell::RefCell;
use hal::digital::OutputPin;

/// Output pin of the shift register
pub struct ShiftRegisterPin<'a, Pin1: 'a, Pin2: 'a, Pin3: 'a>
    where Pin1: OutputPin,
          Pin2: OutputPin,
          Pin3: OutputPin,
{
    shift_register: &'a ShiftRegister<Pin1, Pin2, Pin3>,
    index: usize,
}

impl<'a, Pin1, Pin2, Pin3> ShiftRegisterPin<'a, Pin1, Pin2, Pin3>
    where Pin1: OutputPin,
          Pin2: OutputPin,
          Pin3: OutputPin,
{
    fn new(shift_register: &'a ShiftRegister<Pin1, Pin2, Pin3>, index: usize) -> Self {
        ShiftRegisterPin { shift_register, index }
    }
}

impl<'a, Pin1, Pin2, Pin3> OutputPin for ShiftRegisterPin<'a, Pin1, Pin2, Pin3>
    where Pin1: OutputPin,
          Pin2: OutputPin,
          Pin3: OutputPin,
{
    fn set_low(&mut self) {
        self.shift_register.update(self.index, false);
    }

    fn set_high(&mut self) {
        self.shift_register.update(self.index, true);
    }
}

/// Serial-in parallel-out shift register
pub struct ShiftRegister<Pin1, Pin2, Pin3>
    where Pin1: OutputPin,
          Pin2: OutputPin,
          Pin3: OutputPin,
{
    clock: RefCell<Pin1>,
    latch: RefCell<Pin2>,
    data: RefCell<Pin3>,
    output_state: RefCell<[bool; 8]>,
}

impl<Pin1, Pin2, Pin3> ShiftRegister<Pin1, Pin2, Pin3>
    where Pin1: OutputPin,
          Pin2: OutputPin,
          Pin3: OutputPin,
{
    /// Creates a new SIPO shift register from clock, latch, and data output pins
    pub fn new(clock: Pin1, latch: Pin2, data: Pin3) -> Self {
        ShiftRegister {
            clock: RefCell::new(clock),
            latch: RefCell::new(latch),
            data: RefCell::new(data),
            output_state: RefCell::new([false; 8]),
        }
    }
    /// Sets the value of the shift register output at `index` to value `command`
    pub fn update(&self, index: usize, command: bool) {
        self.output_state.borrow_mut()[index] = command;
        let output_state = self.output_state.borrow();
        self.latch.borrow_mut().set_low();

        for i in 1..=8 {
            if output_state[output_state.len()-i] {self.data.borrow_mut().set_high();}
                else {self.data.borrow_mut().set_low();}
            self.clock.borrow_mut().set_high();
            self.clock.borrow_mut().set_low();
        }

        self.latch.borrow_mut().set_high();
    }
    /// Get embedded-hal output pins to control the shift register outputs
    pub fn decompose(&self) -> [ShiftRegisterPin<Pin1, Pin2, Pin3>; 8] {
        [
            ShiftRegisterPin::new(self, 0),
            ShiftRegisterPin::new(self, 1),
            ShiftRegisterPin::new(self, 2),
            ShiftRegisterPin::new(self, 3),
            ShiftRegisterPin::new(self, 4),
            ShiftRegisterPin::new(self, 5),
            ShiftRegisterPin::new(self, 6),
            ShiftRegisterPin::new(self, 7),
        ]
    }
    /// Consume the shift register and return the original clock, latch, and data output pins
    pub fn release(self) -> (Pin1, Pin2, Pin3) {
        let Self{clock, latch, data, output_state: _} = self;
        (clock.into_inner(), latch.into_inner(), data.into_inner())
    }
}
