use std::cell::RefCell;

use proptest::prelude::*;
use rotary_encoder_hal::{Direction, Rotary};

#[cfg(not(feature = "embedded-hal-alpha"))]
use embedded_hal::digital::InputPin;

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

fn direction_strategy() -> impl Strategy<Value = Direction> {
    prop_oneof![
        Just(Direction::Clockwise),
        Just(Direction::CounterClockwise),
    ]
}

#[derive(Clone, Debug)]
enum Start {
    A0B0(Direction),
    A1B0(Direction),
    A0B1(Direction),
    A1B1(Direction),
}

fn start_strategy() -> impl Strategy<Value = Start> {
    prop_oneof![
        direction_strategy().prop_map(Start::A0B0),
        direction_strategy().prop_map(Start::A1B0),
        direction_strategy().prop_map(Start::A0B1),
        direction_strategy().prop_map(Start::A1B1),
    ]
}

#[derive(Clone, Debug)]
enum Op {
    Advance,
    Reverse,
    Bounce,
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![Just(Op::Advance), Just(Op::Reverse), Just(Op::Bounce)]
}

fn rev(d: Direction) -> Direction {
    match d {
        R => L,
        L => R,
        _ => unreachable!(),
    }
}

fn run_prop_test(start: Start, ops: &[Op]) {
    let (mut a, mut b, mut dir) = match start {
        Start::A0B0(d) => (0, 0, d),
        Start::A1B0(d) => (1, 0, d),
        Start::A0B1(d) => (0, 1, d),
        Start::A1B1(d) => (1, 1, d),
    };
    let mut recent_reverse = false;
    let (mut v_a, mut v_b, mut expected) = (Vec::new(), Vec::new(), Vec::new());
    v_a.push(a);
    v_b.push(b);
    expected.push(N);
    for op in ops {
        match op {
            Op::Advance => {
                let d;
                (a, b, d) = match (a, b, dir) {
                    (0, 0, R) => (1, 0, N), // 0x81
                    (0, 1, R) => (0, 0, N), // 0xE8
                    (1, 0, R) => (1, 1, R), // 0x17
                    (1, 1, R) => (0, 1, N), // 0x7E
                    (0, 0, L) => (0, 1, N), // 0x42
                    (0, 1, L) => (1, 1, L), // 0x2B
                    (1, 0, L) => (0, 0, N), // 0xD4
                    (1, 1, L) => (1, 0, N), // 0xBD
                    _ => unreachable!(),
                };
                v_a.push(a);
                v_b.push(b);
                expected.push(if recent_reverse && expected.len() > 1 {
                    N
                } else {
                    d
                });
                recent_reverse = false;
            }
            Op::Reverse => {
                dir = rev(dir);
                recent_reverse = !recent_reverse;
            }
            Op::Bounce => {
                if expected.len() <= 1 {
                    continue;
                }
                let d = if recent_reverse { rev(dir) } else { dir };
                let (a1, a2, b1, b2) = match (a, b, d) {
                    (0, 0, R) => (0, 0, 1, 0),
                    (0, 1, R) => (1, 0, 1, 1),
                    (1, 0, R) => (0, 1, 0, 0),
                    (1, 1, R) => (1, 1, 0, 1),
                    (0, 0, L) => (1, 0, 0, 0),
                    (0, 1, L) => (0, 0, 0, 1),
                    (1, 0, L) => (1, 1, 1, 0),
                    (1, 1, L) => (0, 1, 1, 1),
                    _ => unreachable!(),
                };
                v_a.push(a1);
                v_a.push(a2);
                v_b.push(b1);
                v_b.push(b2);
                expected.push(N);
                expected.push(N);
            }
        }
    }
    run(v_a.as_slice(), v_b.as_slice(), expected.as_slice());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    #[cfg(feature = "table-decoder")]
    fn prop_test(start in start_strategy(), ops in prop::collection::vec(op_strategy(), 0..1000)) {
        run_prop_test(start, ops.as_slice());
    }
}
