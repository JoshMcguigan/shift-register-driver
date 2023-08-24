//! Serial-in parallel-out shift register
extern crate alloc;

use alloc::sync::Arc;
#[cfg(not(feature = "freertos"))]
use core::cell::RefCell;
use core::mem::{self, MaybeUninit};

use freertos_rust::*;

use crate::hal::digital::v2::OutputPin;

#[cfg(not(feature = "freertos"))]
trait ShiftRegisterInternal {
    fn update(&self, index: usize, command: bool) -> Result<(), ()>;
}

#[cfg(feature = "freertos")]
trait ShiftRegisterInternal: Sync {
    fn update(&self, index: usize, command: bool) -> Result<(), ()>;
}

/// Output pin of the shift register
pub struct ShiftRegisterPin<'a>
{
    shift_register: &'a dyn ShiftRegisterInternal,
    index: usize,
}

#[cfg(feature = "freertos")]
unsafe impl<'a> Sync for ShiftRegisterPin<'a> {}

impl<'a> ShiftRegisterPin<'a>
{
    fn new(shift_register: &'a dyn ShiftRegisterInternal, index: usize) -> Self {
        ShiftRegisterPin { shift_register, index }
    }
}

impl OutputPin for ShiftRegisterPin<'_>
{
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.shift_register.update(self.index, false)?;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.shift_register.update(self.index, true)?;
        Ok(())
    }
}

#[cfg(not(feature = "freertos"))]
macro_rules! ShiftRegisterBuilder {
    ($name: ident, $size: expr) => {
        /// Serial-in parallel-out shift register
        pub struct $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin
        {
            clock: RefCell<Pin1>,
            latch: RefCell<Pin2>,
            data: RefCell<Pin3>,
            output_state: RefCell<[bool; $size]>,
        }

        impl<Pin1, Pin2, Pin3> ShiftRegisterInternal for $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin
        {
            /// Sets the value of the shift register output at `index` to value `command`
            fn update(&self, index: usize, command: bool) -> Result<(), ()>{
                self.output_state.borrow_mut()[index] = command;
                let output_state = self.output_state.borrow();
                self.latch.borrow_mut().set_low().map_err(|_e| ())?;

                for i in 1..=output_state.len() {
                    if output_state[output_state.len() - i] {
                        self.data.borrow_mut().set_high().map_err(|_e| ())?;
                    } else {
                        self.data.borrow_mut().set_low().map_err(|_e| ())?;
                    }
                    self.clock.borrow_mut().set_high().map_err(|_e| ())?;
                    self.clock.borrow_mut().set_low().map_err(|_e| ())?;
                }

                self.latch.borrow_mut().set_high().map_err(|_e| ())?;
                Ok(())
            }
        }


        impl<Pin1, Pin2, Pin3> $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin
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
            pub fn decompose(&self) ->  [ShiftRegisterPin; $size] {

                // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
                // safe because the type we are claiming to have initialized here is a
                // bunch of `MaybeUninit`s, which do not require initialization.
                let mut pins:  [MaybeUninit<ShiftRegisterPin>; $size] = unsafe {
                    MaybeUninit::uninit().assume_init()
                };

                // Dropping a `MaybeUninit` does nothing, so if there is a panic during this loop,
                // we have a memory leak, but there is no memory safety issue.
                for (index, elem) in pins.iter_mut().enumerate() {
                    elem.write(ShiftRegisterPin::new(self, index));
                }

                // Everything is initialized. Transmute the array to the
                // initialized type.
                unsafe { mem::transmute::<_, [ShiftRegisterPin; $size]>(pins) }
            }

            /// Consume the shift register and return the original clock, latch, and data output pins
            pub fn release(self) -> (Pin1, Pin2, Pin3) {
                let Self{clock, latch, data, output_state: _} = self;
                (clock.into_inner(), latch.into_inner(), data.into_inner())
            }
        }

    }
}

#[cfg(feature = "freertos")]
macro_rules! ShiftRegisterBuilder {
    ($name: ident, $size: expr) => {
        /// Serial-in parallel-out shift register
        pub struct $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin,
                  Pin1: core::marker::Send,
                  Pin2: core::marker::Send,
                  Pin3: core::marker::Send,
                  Pin1: core::marker::Sync,
                  Pin2: core::marker::Sync,
                  Pin3: core::marker::Sync
        {
            clock: Arc<Mutex<Pin1>>,
            latch: Arc<Mutex<Pin2>>,
            data: Arc<Mutex<Pin3>>,
            output_state:  Arc<Mutex<[bool; $size]>>,
        }

        impl<Pin1, Pin2, Pin3> ShiftRegisterInternal for $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin,
                  Pin1: core::marker::Send,
                  Pin2: core::marker::Send,
                  Pin3: core::marker::Send,
                  Pin1: core::marker::Sync,
                  Pin2: core::marker::Sync,
                  Pin3: core::marker::Sync
        {
            /// Sets the value of the shift register output at `index` to value `command`
            fn update(&self, index: usize, command: bool) -> Result<(), ()>{

                if let Ok(mut output_state) = self.output_state.lock(Duration::ms(1)){
                    output_state[index] = command;
                };

                if let Ok(output_state) = self.output_state.lock(Duration::ms(1)){
                    if let Ok(mut latch) = self.latch.lock(Duration::ms(1)){
                        latch.set_low().map_err(|_e| ())?;
                    };

                    for i in 1..=output_state.len() {
                        if output_state[output_state.len() - i] {
                            if let Ok(mut data) = self.data.lock(Duration::ms(1)){
                                data.set_high().map_err(|_e| ())?;
                            };
                        } else {
                            if let Ok(mut data) = self.data.lock(Duration::ms(1)){
                                data.set_low().map_err(|_e| ())?;
                            };
                        }
                        if let Ok(mut clock) = self.clock.lock(Duration::ms(1)){
                            clock.set_high().map_err(|_e| ())?;
                        };
                        if let Ok(mut clock) = self.clock.lock(Duration::ms(1)){
                            clock.set_low().map_err(|_e| ())?;
                        };
                    }

                    if let Ok(mut latch) = self.latch.lock(Duration::ms(1)){
                        latch.set_high().map_err(|_e| ())?;
                    };
                    Ok(())
                } else {
                    Err(())
                }
            }
        }


        impl<Pin1, Pin2, Pin3> $name<Pin1, Pin2, Pin3>
            where Pin1: OutputPin,
                  Pin2: OutputPin,
                  Pin3: OutputPin,
                  Pin1: core::marker::Send,
                  Pin2: core::marker::Send,
                  Pin3: core::marker::Send,
                  Pin1: core::marker::Sync,
                  Pin2: core::marker::Sync,
                  Pin3: core::marker::Sync
        {
            /// Creates a new SIPO shift register from clock, latch, and data output pins
            pub fn new(clock: Pin1, latch: Pin2, data: Pin3) -> Self {
                $name {
                    clock: Arc::new(Mutex::new(clock).expect("Mutex creation failed")),
                    latch: Arc::new(Mutex::new(latch).expect("Mutex creation failed")),
                    data: Arc::new(Mutex::new(data).expect("Mutex creation failed")),
                    output_state: Arc::new(Mutex::new([false; $size]).expect("Mutex creation failed")),
                }
            }

            /// Get embedded-hal output pins to control the shift register outputs
            pub fn decompose(&self) ->  [ShiftRegisterPin; $size] {

                // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
                // safe because the type we are claiming to have initialized here is a
                // bunch of `MaybeUninit`s, which do not require initialization.
                let mut pins:  [MaybeUninit<ShiftRegisterPin>; $size] = unsafe {
                    MaybeUninit::uninit().assume_init()
                };

                // Dropping a `MaybeUninit` does nothing, so if there is a panic during this loop,
                // we have a memory leak, but there is no memory safety issue.
                for (index, elem) in pins.iter_mut().enumerate() {
                    elem.write(ShiftRegisterPin::new(self, index));
                }

                // Everything is initialized. Transmute the array to the
                // initialized type.
                unsafe { mem::transmute::<_, [ShiftRegisterPin; $size]>(pins) }
            }

            // /// Consume the shift register and return the original clock, latch, and data output pins
            // pub fn release(self) -> (Pin1, Pin2, Pin3) {
            //     let Self{clock, latch, data, output_state: _} = self;
            //     (clock.into_inner(), latch.into_inner(), data.into_inner())
            // }
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
