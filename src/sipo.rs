//! Serial-in parallel-out shift register

use core::cell::RefCell;
use hal::digital::v2::OutputPin;
use core::mem;
use core::ptr;

trait ShiftRegisterInternal {
    fn update(&self, index: usize, command: bool);
}

/// Output pin of the shift register
pub struct ShiftRegisterPin<'a>
{
    shift_register: &'a ShiftRegisterInternal,
    index: usize,
}

impl<'a> ShiftRegisterPin<'a>
{
    fn new(shift_register: &'a ShiftRegisterInternal, index: usize) -> Self {
        ShiftRegisterPin { shift_register, index }
    }
}

impl<'a> OutputPin for ShiftRegisterPin<'a>
{
    fn set_low(&mut self) {
        self.shift_register.update(self.index, false);
    }

    fn set_high(&mut self) {
        self.shift_register.update(self.index, true);
    }
}

macro_rules! ShiftRegisterBuilder {
    ($name: ident, $size: expr) => {
        /// Serial-in parallel-out shift register
        pub struct $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin,
        {
            clock: RefCell<Pin1>,
            latch: RefCell<Pin2>,
            data: RefCell<Pin3>,
            output_state: RefCell<[bool; $size]>,
        }

        impl<Pin1, Pin2, Pin3> ShiftRegisterInternal for $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin,
        {
            /// Sets the value of the shift register output at `index` to value `command`
            fn update(&self, index: usize, command: bool) {
                self.output_state.borrow_mut()[index] = command;
                let output_state = self.output_state.borrow();
                self.latch.borrow_mut().set_low();

                for i in 1..=output_state.len() {
                    if output_state[output_state.len()-i] {self.data.borrow_mut().set_high();}
                        else {self.data.borrow_mut().set_low();}
                    self.clock.borrow_mut().set_high();
                    self.clock.borrow_mut().set_low();
                }

                self.latch.borrow_mut().set_high();
            }
        }


        impl<Pin1, Pin2, Pin3> $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
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
            pub fn decompose(&self) -> [ShiftRegisterPin; $size] {
                let mut pins: [ShiftRegisterPin; $size];

                unsafe {
                    pins = mem::uninitialized();
                    for (index, elem) in pins[..].iter_mut().enumerate() {
                        ptr::write(elem, ShiftRegisterPin::new(self, index));
                    }
                }

                pins
            }

            /// Consume the shift register and return the original clock, latch, and data output pins
            pub fn release(self) -> (Pin1, Pin2, Pin3) {
                let Self{clock, latch, data, output_state: _} = self;
                (clock.into_inner(), latch.into_inner(), data.into_inner())
            }
        }

    }
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
