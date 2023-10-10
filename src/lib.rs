#![doc(html_root_url = "https://docs.rs/rotary-encoder-hal/0.5.0")]
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

#[cfg(not(feature = "embedded-hal-alpha"))]
use embedded_hal::digital::v2::InputPin;

#[cfg(feature = "embedded-hal-alpha")]
use embedded_hal_alpha::digital::blocking::InputPin;

/// Holds current/old state and both [`InputPin`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html)
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rotary<A, B> {
    pin_a: A,
    pin_b: B,
    state: u8,
}

/// The encoder direction is either `Clockwise`, `CounterClockwise`, or `None`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// A clockwise turn
    Clockwise,
    /// A counterclockwise turn
    CounterClockwise,
    /// No change
    None,
}

#[cfg(not(feature = "table-decoder"))]
impl From<u8> for Direction {
    fn from(s: u8) -> Self {
        match s {
            0b0001 | 0b0111 | 0b1000 | 0b1110 => Direction::Clockwise,
            0b0010 | 0b0100 | 0b1011 | 0b1101 => Direction::CounterClockwise,
            _ => Direction::None,
        }
    }
}

#[cfg(feature = "table-decoder")]
impl From<u8> for Direction {
    fn from(s: u8) -> Self {
        match s {
            0x17 => Direction::CounterClockwise,
            0x2b => Direction::Clockwise,
            _ => Direction::None,
        }
    }
}

impl<A, B> Rotary<A, B>
where
    A: InputPin,
    B: InputPin,
{
    /// Accepts two [`InputPin`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/digital/v2/trait.InputPin.html)s, these will be read on every `update()`
    pub fn new(pin_a: A, pin_b: B) -> Self {
        Self {
            pin_a,
            pin_b,
            state: 0u8,
        }
    }

    #[cfg(not(feature = "table-decoder"))]
    /// Call `update` to evaluate the next state of the encoder, propagates errors from `InputPin` read
    pub fn update(&mut self) -> Result<Direction, Either<A::Error, B::Error>> {
        // use mask to get previous state value
        let mut s = self.state & 0b11;

        let (a_is_low, b_is_low) = (self.pin_a.is_low(), self.pin_b.is_low());

        // move in the new state
        if a_is_low.map_err(Either::Left)? {
            s |= 0b100;
        }
        if b_is_low.map_err(Either::Right)? {
            s |= 0b1000;
        }
        // move new state in
        self.state = s >> 2;
        Ok(s.into())
    }

    /// Returns a reference to the first pin. Can be used to clear interrupt.
    pub fn pin_a(&mut self) -> &mut A {
        &mut self.pin_a
    }

    /// Returns a reference to the second pin. Can be used to clear interrupt.
    pub fn pin_b(&mut self) -> &mut B {
        &mut self.pin_b
    }

    /// Returns a reference to both pins. Can be used to clear interrupt.
    pub fn pins(&mut self) -> (&mut A, &mut B) {
        (&mut self.pin_a, &mut self.pin_b)
    }

    /// Consumes this `Rotary`, returning the underlying pins `A` and `B`.
    pub fn into_inner(self) -> (A, B) {
        (self.pin_a, self.pin_b)
    }

    #[cfg(feature = "table-decoder")]
    /// Call `update` to evaluate the next state of the encoder, propagates errors from `InputPin` read
    pub fn update(&mut self) -> Result<Direction, Either<A::Error, B::Error>> {
        let (a_is_high, b_is_high) = (self.pin_a.is_high(), self.pin_b.is_high());

        // Implemented after https://www.best-microcontroller-projects.com/rotary-encoder.html
        let mut prev_next = (self.state << 2) & 0xF;
        if a_is_high.map_err(Either::Left)? {
            prev_next |= 0x01;
        }
        if b_is_high.map_err(Either::Right)? {
            prev_next |= 0x02;
        }

        match prev_next {
            /*valid cases*/
            1 | 2 | 4 | 7 | 8 | 11 | 13 | 14 => {
                let result = (self.state & 0xF0) | prev_next;
                self.state = prev_next << 4 | prev_next;

                Ok(result.into())
            }
            /*Invalid cases */
            0 | 3 | 5 | 6 | 9 | 10 | 12 | 15 => {
                self.state = self.state & 0xF0 | prev_next;

                Ok(Direction::None)
            }
            /* let the compiler help us ensure we've covered them all */
            0x10..=0xFF => Ok(Direction::None),
        }
    }
}
