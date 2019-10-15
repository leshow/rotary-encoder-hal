# rotary-encoder-hal

A simple, platform agnostic rotary encoder library.

```rust
#![no_std]
#![no_main]

extern crate panic_itm;

use cortex_m::iprintln;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::{hio, hprintln};
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
