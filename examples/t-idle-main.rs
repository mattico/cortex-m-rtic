#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::debug;

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (init::LateResources {}, init::Monotonics())
    }

    #[idle]
    fn taskmain(_: taskmain::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }
}
