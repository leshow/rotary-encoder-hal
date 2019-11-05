#![doc(html_root_url = "https://docs.rs/rotary-encoder-hal/0.3.0")]
//! # rotary-encoder-hal
//!
//! A platform agnostic rotary encoder library
//!
//! Built using [`embedded-hal`] traits
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/0.2

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use either::Either;
use embedded_hal as hal;
use hal::{digital::v2::InputPin, Direction, Qei};

/// Holds current/old state and both [`InputPin`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rotary<A, B> {
    pin_a: A,
    pin_b: B,
    state: u8,
    count: isize,
}

/// Takes a `u8` representing the current state of the encoder and returns a `Direction`
pub fn from_u8(s: u8) -> Option<Direction> {
    match s {
        0b0001 | 0b0111 | 0b1000 | 0b1110 => Some(Direction::Upcounting),
        0b0010 | 0b0100 | 0b1011 | 0b1101 => Some(Direction::Downcounting),
        _ => None,
    }
}

impl<A, B> Qei for Rotary<A, B> {
    type Count = isize;
    fn count(&self) -> Self::Count {
        self.count
    }

    fn direction(&self) -> Direction {
        unimplemented!()
    }
}

impl<A, B> Rotary<A, B>
where
    A: InputPin,
    B: InputPin,
{
    /// Accepts two `InputPin`s, these will be read on every `update()`
    /// [InputPin]: https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html
    pub fn new(pin_a: A, pin_b: B) -> Self {
        Self {
            pin_a,
            pin_b,
            state: 0u8,
            count: 0,
        }
    }
    /// Call `update` to evaluate the next state of the encoder, propagates errors from `InputPin` read
    pub fn update(&mut self) -> Result<Option<Direction>, Either<A::Error, B::Error>> {
        // use mask to get previous state value
        let mut s = self.state & 0b11;
        // move in the new state
        if self.pin_a.is_low().map_err(Either::Left)? {
            s |= 0b100;
        }
        if self.pin_b.is_low().map_err(Either::Right)? {
            s |= 0b1000;
        }
        // move new state in
        self.state = s >> 2;
        let dir = from_u8(s);
        if let Some(d) = dir {
            match d {
                Direction::Downcounting => {
                    self.count = self.count.wrapping_sub(1);
                }
                Direction::Upcounting => {
                    self.count = self.count.wrapping_add(1);
                }
            }
        }
        Ok(dir)
    }
}
