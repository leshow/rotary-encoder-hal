use std::cell::RefCell;

use rotary_encoder_hal::{Direction, Rotary};

#[cfg(not(feature = "embedded-hal-alpha"))]
use embedded_hal::digital::v2::InputPin;

#[cfg(feature = "embedded-hal-alpha")]
use embedded_hal_alpha::digital::blocking::InputPin;

struct FakeInputPin<I>(RefCell<I>);

impl<I: Iterator<Item = u8>> InputPin for FakeInputPin<I> {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        if let Some(pd) = self.0.borrow_mut().next() {
            return Ok(pd == 1);
        }
        Err(())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        if let Some(pd) = self.0.borrow_mut().next() {
            return Ok(pd == 0);
        }
        Err(())
    }
}

fn run(v_a: &[u8], v_b: &[u8], expected: &[Direction]) {
    assert_eq!(expected.len(), v_a.len());
    assert_eq!(expected.len(), v_b.len());

    let f_a = FakeInputPin(RefCell::new(v_a.into_iter().cloned()));
    let f_b = FakeInputPin(RefCell::new(v_b.into_iter().cloned()));
    let mut rotary = Rotary::new(f_a, f_b);
    for (i, &dir) in expected.iter().enumerate() {
        assert_eq!(dir, rotary.update().unwrap(), "index: {}", i);
    }
}

const R: Direction = Direction::CounterClockwise;
const L: Direction = Direction::Clockwise;
const N: Direction = Direction::None;

#[test]
#[cfg(not(feature = "table-decoder"))]
fn test() {
    run(
        &[0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[N, R, R, R, R, R, R, R, R],
    );
    run(
        &[0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[N, L, L, L, L, L, L, L, L],
    );
    run(
        &[0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[N, R, L, R, R, R, R, R, R, R, R],
    );
    run(
        &[0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[N, L, R, L, L, L, L, L, L, L, L],
    );
}

#[test]
#[cfg(feature = "table-decoder")]
fn test() {
    run(
        &[0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[N, N, R, N, N, N, R, N, N],
    );
    run(
        &[0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[N, N, L, N, N, N, L, N, N],
    );
    run(
        &[0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[N, N, N, N, R, N, N, N, R, N, N],
    );
    run(
        &[0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0],
        &[0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0],
        &[N, N, N, N, L, N, N, N, L, N, N],
    );
    run(
        &[0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0],
        &[0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0],
        &[N, N, R, N, N, N, N, N, R, N, N],
    );
    run(
        &[0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0],
        &[0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0],
        &[N, N, L, N, N, N, N, N, L, N, N],
    );
}
