//! NUCLEO-H753ZI — TB6612FNG DC motor driver example
//!
//! Wiring (NUCLEO-H753ZI connectors → TB6612FNG):
//!
//!  All control pins are on CN7, rows 2-4 (easy to wire):
//!
//!  MCU pin │ CN7 pin │ TB6612FNG │ Notes
//!  ────────┼─────────┼───────────┼────────────────────────────────
//!  PB9     │  pin 4  │ PWMA      │ TIM4_CH4 (AF2), 20 kHz PWM
//!  PB15    │  pin 3  │ AIN1      │ Direction bit 1
//!  PA15    │  pin 9  │ AIN2      │ Direction bit 2
//!  PB12    │  pin 7  │ STBY      │ Active-high enable
//!  GND     │  pin 8  │ GND / VM- │ Adjacent to STBY pin
//!  +12 V   │ external│ VM        │ Motor supply (up to 15 V)
//!  +3.3 V  │ board   │ VCC       │ Logic supply

#![no_std]
#![no_main]

mod motor_driver;

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::{pac, prelude::*};

use crate::motor_driver::Motor;

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Power configuration (required before touching RCC on H7)
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // System clock: 400 MHz
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(400.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    // Split GPIO bank — all pins are on GPIOB / CN7
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    // PB13 is held by the Ethernet PHY, so we'll use PB5 instead for AIN2 — it's adjacent to PB9 (PWMA) and PB12 (STBY) on CN7
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    // STBY: CN7 pin 7 — active-high enable
    let mut stby = gpiob.pb12.into_push_pull_output();

    // AIN1 (CN7 pin 3) / AIN2 (CN7 pin 6) — PB13 is RMII_TXD1, driven by on-board Ethernet PHY
    let ain1 = gpiob.pb15.into_push_pull_output();
    let ain2 = gpioa.pa15.into_push_pull_output();

    // PWMA: PB9 (CN7 pin 4) → TIM4_CH4 (AF2), 20 kHz
    let pwm_pin = gpiob.pb9.into_alternate::<2>();
    let pwm = dp
        .TIM4
        .pwm(pwm_pin, 20.kHz(), ccdr.peripheral.TIM4, &ccdr.clocks);

    // Build motor driver (also calls pwm.enable() internally)
    let mut motor = Motor::new(ain1, ain2, pwm);

    // SysTick delay — must come after the last &ccdr.clocks borrow above
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // Bring TB6612FNG out of standby
    stby.set_high();
    delay.delay_ms(1u32);

    rtt_init_print!();
    loop {
        rprintln!("Forward 50 % for 2 s");
        motor.forward(50);
        delay.delay_ms(2_000u32);

        rprintln!("Short brake for 500 ms");
        motor.brake();
        delay.delay_ms(500u32);

        rprintln!("Reverse 30 % for 2 s");
        motor.reverse(30);
        delay.delay_ms(2_000u32);

        rprintln!("Short brake for 500 ms");
        motor.brake();
        delay.delay_ms(500u32);
    }
}
