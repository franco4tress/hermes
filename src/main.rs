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
//!  PB11    │  pin 6  │ AIN2      │ Direction bit 2  (PB13 is RMII_TXD1, held by Ethernet PHY)
//!  PB12    │  pin 7  │ STBY      │ Active-high enable
//!  GND     │  pin 8  │ GND / VM- │ Adjacent to STBY pin
//!  +12 V   │ external│ VM        │ Motor supply (up to 15 V)
//!  +3.3 V  │ board   │ VCC       │ Logic supply

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::{PwmPin, digital::v2::OutputPin};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::{pac, prelude::*};

// ─────────────────────────────────────────────────────────────────────────────
// TB6612FNG single-channel abstraction
// ─────────────────────────────────────────────────────────────────────────────

/// Controls one motor channel of a TB6612FNG.
///
/// Truth table (STBY must be high externally):
///   AIN1 H, AIN2 L, PWM = duty  →  forward
///   AIN1 L, AIN2 H, PWM = duty  →  reverse
///   AIN1 H, AIN2 H              →  short brake
///   AIN1 L, AIN2 L, PWM = 0    →  coast / stop
struct Motor<AIN1, AIN2, PWM> {
    ain1: AIN1,
    ain2: AIN2,
    pwm: PWM,
    max_duty: u16,
}

impl<AIN1, AIN2, PWM> Motor<AIN1, AIN2, PWM>
where
    AIN1: OutputPin,
    AIN2: OutputPin,
    PWM: PwmPin<Duty = u16>,
{
    fn new(ain1: AIN1, ain2: AIN2, mut pwm: PWM) -> Self {
        let max_duty = pwm.get_max_duty();
        pwm.enable();
        Motor {
            ain1,
            ain2,
            pwm,
            max_duty,
        }
    }

    /// Drive forward at `speed_pct` (0–100 %).
    fn forward(&mut self, speed_pct: u8) {
        let duty = (self.max_duty as u32 * speed_pct.min(100) as u32 / 100) as u16;
        let _ = self.ain1.set_high();
        let _ = self.ain2.set_low();
        self.pwm.set_duty(duty);
    }

    /// Drive in reverse at `speed_pct` (0–100 %).
    fn reverse(&mut self, speed_pct: u8) {
        let duty = (self.max_duty as u32 * speed_pct.min(100) as u32 / 100) as u16;
        let _ = self.ain1.set_low();
        let _ = self.ain2.set_high();
        self.pwm.set_duty(duty);
    }

    /// Short-circuit brake — both outputs pulled low through the H-bridge.
    fn brake(&mut self) {
        let _ = self.ain1.set_high();
        let _ = self.ain2.set_high();
    }

    /// Coast — outputs go high-impedance, motor spins down on its own.
    #[allow(dead_code)]
    fn coast(&mut self) {
        self.pwm.set_duty(0);
        let _ = self.ain1.set_low();
        let _ = self.ain2.set_low();
    }
}

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
        rprintln!("Forward 75 % for 2 s");
        motor.forward(75);
        delay.delay_ms(2_000u32);

        rprintln!("Short brake for 500 ms");
        motor.brake();
        delay.delay_ms(500u32);

        rprintln!("Reverse 50 % for 2 s");
        motor.reverse(50);
        delay.delay_ms(2_000u32);

        rprintln!("Short brake for 500 ms");
        motor.brake();
        delay.delay_ms(500u32);
    }
}
