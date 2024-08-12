# rotary-encoder-hal

[![Crate](https://img.shields.io/crates/v/rotary-encoder-hal.svg)](https://crates.io/crates/rotary-encoder-hal)
[![API](https://docs.rs/rotary-encoder-hal/badge.svg)](https://docs.rs/rotary-encoder-hal)

A simple, platform agnostic rotary encoder library.

An alternate decoder algorithm that is more tolerant of
noise is enabled by the Cargo feature `table-decoder`.  It
follows the discussion of noisy decoding
[here](https://www.best-microcontroller-projects.com/rotary-encoder.html).


You can call `update` from an ISR. You may get many spurious
interrupts from switch bounce, though the algorithm handles
them appropriately. When polling `update`, a poll time of
about 1 ms seems to work well.

## Examples

```rust
#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use hal::{delay::Delay, prelude::*, stm32};
use stm32f3xx_hal as hal;

use rotary_encoder_hal::{Direction, Rotary};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb);
    let pin_a = gpiob
        .pb10
        .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr);
    let pin_b = gpiob
        .pb11
        .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr);

    let mut enc = Rotary::new(pin_a, pin_b);
    let mut pos: isize = 0;

    loop {
        match enc.update().unwrap() {
            Direction::Clockwise => {
                pos += 1;
            }
            Direction::CounterClockwise => {
                pos -= 1;
            }
            Direction::None => {}
        }
    }
}
```
