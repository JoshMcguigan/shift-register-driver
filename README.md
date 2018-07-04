# shift-register-driver [![Docs](https://img.shields.io/crates/v/shift-register-driver.svg)](https://crates.io/crates/shift-register-driver) [![Docs](https://docs.rs/shift-register-driver/badge.svg)](https://docs.rs/shift-register-driver)

> Platform agnostic driver for shift register's built using the embedded-hal

## What works

- Controlling outputs through serial-in parallel-out shift registers with 8 outputs

## TODO

- [ ] Add parallel-out serial-in shift register support
- [ ] Support chained shift registers for more than 8 IO

## Example

```rust
    use shift_register_driver::sipo::ShiftRegister;
    let shift_register = ShiftRegister::new(clock, latch, data);
    {
        let mut outputs = shift_register.decompose();

        for i in 0..8 {
            // Optionally control the pins directly through the shift_register struct
            //      or individually by using the decompose method
            // shift_register.update(i, true);

            outputs[i].set_high();
            delay.delay_ms(300u32);
        }

        for i in 0..8 {
            outputs[7-i].set_low();
            delay.delay_ms(300u32);
        }

    }
    // shift_register.release() can optionally be used when the shift register is no longer needed
    //      in order to regain ownership of the original GPIO pins
    let (clock, latch, data) = shift_register.release();
```
    
## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
