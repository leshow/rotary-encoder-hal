#![doc(html_root_url = "https://docs.rs/rotary-encoder-hal/0.2.1")]
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
use hal::digital::v2::InputPin;

/// Holds current/old state and both [`InputPin`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rotary<A, B> {
    pin_a: A,
    pin_b: B,
    state: u8,
}

/// The encoder direction is either `Clockwise`, `CounterClockwise`, or `None`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// A clockwise turn
    Clockwise,
    /// A counterclockwise turn
    CounterClockwise,
    /// No change
    None,
}

impl From<u8> for Direction {
    fn from(s: u8) -> Self {
        match s {
            0b0001 | 0b0111 | 0b1000 | 0b1110 => Direction::Clockwise,
            0b0010 | 0b0100 | 0b1011 | 0b1101 => Direction::CounterClockwise,
            _ => Direction::None,
        }
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
        }
    }
    /// Call `update` to evaluate the next state of the encoder, propagates errors from `InputPin` read
    pub fn update(&mut self) -> Result<Direction, Either<A::Error, B::Error>> {
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
        Ok(s.into())
    }
}
