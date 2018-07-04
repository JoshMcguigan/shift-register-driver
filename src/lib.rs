//! Platform agnostic driver for shift register's built using the embedded-hal

#![allow(dead_code)]
#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

extern crate embedded_hal as hal;

pub mod sipo;
