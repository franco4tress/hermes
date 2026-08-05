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
use stm32h7xx_hal::{pac, prelude::*, pwm};

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

    // Split GPIO bank — all left motor pins are on GPIOB / CN7
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    // PB13 is held by the Ethernet PHY, so we'll use PB5 instead for AIN2 — it's adjacent to PB9 (PWMA) and PB12 (STBY) on CN7
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    
    // All right motor pins are on GPIOE / CN9 and GPIOD / CN10
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);

    // STBY: CN7 pin 7 — active-high enable
    let mut stby_left = gpiob.pb12.into_push_pull_output();

    ///////////////////////////////////////////////////////////////////////////////////////////
    //                             Front Left Motor configuration
    ///////////////////////////////////////////////////////////////////////////////////////////

    // AIN1 (CN7 pin 3) / AIN2 (CN7 pin 6) — PB13 is RMII_TXD1, driven by on-board Ethernet PHY
    let ain1_left = gpiob.pb15.into_push_pull_output();
    let ain2_left = gpioa.pa15.into_push_pull_output();

    // PWMA: PB9 (CN7 pin 4) → TIM4_CH4 (AF2), 20 kHz
    let pwma_pin_left = gpiob.pb9.into_alternate::<2>();

    ///////////////////////////////////////////////////////////////////////////////////////////
    //                             Back Left Motor configuration
    ///////////////////////////////////////////////////////////////////////////////////////////

    // BIN1 (CN7 pin 13) / BIN2 (CN7 pin 19)
    let bin1_left = gpiob.pb4.into_push_pull_output();
    let bin2_left = gpiob.pb5.into_push_pull_output();

    // PWMB: PB8 (CN7 pin 14) → TIM4_CH3 (AF2), 20 kHz
    let pwmb_pin_left = gpiob.pb8.into_alternate::<2>();


    /////////////////////////////////////////////////////////////////////////////////////////////
    //                            Front Right Motor configuration
    /////////////////////////////////////////////////////////////////////////////////////////////
    
    let ain1_right = gpioe.pe6.into_push_pull_output();
    let ain2_right = gpioe.pe3.into_push_pull_output();

    let pwma_pin_right = gpiod.pd12.into_alternate::<2>();

    /////////////////////////////////////////////////////////////////////////////////////////////
    //                            Back Right Motor configuration
    /////////////////////////////////////////////////////////////////////////////////////////////
     
    let bin1_right = gpioe.pe4.into_push_pull_output();
    let bin2_right = gpioe.pe2.into_push_pull_output();

    let pwmb_pin_right = gpiod.pd13.into_alternate::<2>();


    let mut stby_right = gpioe.pe5.into_push_pull_output();

    ////////////////////////////////////////////////////////////////////////////////////////////
    // Build PWM channels
    ////////////////////////////////////////////////////////////////////////////////////////////

    // Configure ALL channels in one call — pass a tuple of pins
    let (pwmb_left, pwma_left, pwmb_right, pwma_right) = dp.TIM4.pwm(
        (pwmb_pin_left, pwma_pin_left, pwmb_pin_right, pwma_pin_right),
        20.kHz(),
        ccdr.peripheral.TIM4,
        &ccdr.clocks,
    );

    // Build motor driver (also calls pwm.enable() internally)
    let mut front_left_motor = Motor::new(ain1_left, ain2_left, pwma_left, motor_driver::Polarity::Forward, 0.0);
    let mut back_left_motor = Motor::new(bin1_left, bin2_left, pwmb_left, motor_driver::Polarity::Forward, 0.0);
    // Reverse the right motors to make the robot drive forward (installation is mirrored)
    let mut front_right_motor = Motor::new(ain1_right, ain2_right, pwma_right, motor_driver::Polarity::Reverse, 0.0);
    let mut back_right_motor = Motor::new(bin1_right, bin2_right, pwmb_right, motor_driver::Polarity::Reverse, 0.0);

    ////////////////////////////////////////////////////////////////////////////////////////////

    // SysTick delay — must come after the last &ccdr.clocks borrow above
    let mut delay = cp.SYST.delay(ccdr.clocks);

    // Bring TB6612FNG out of standby
    stby_left.set_high();
    stby_right.set_high();
    delay.delay_ms(1u32);

    rtt_init_print!();
    loop {
        front_right_motor.set_polarity(motor_driver::Polarity::Reverse);
        back_right_motor.set_polarity(motor_driver::Polarity::Reverse);
        rprintln!("Forward 50 % for 2 s");
        front_left_motor.set_polarity(motor_driver::Polarity::Forward);
        back_left_motor.set_polarity(motor_driver::Polarity::Forward);
        front_right_motor.set_polarity(motor_driver::Polarity::Reverse);
        back_right_motor.set_polarity(motor_driver::Polarity::Reverse);
        front_left_motor.drive(50);
        back_left_motor.drive(50);
        front_right_motor.drive(50);
        back_right_motor.drive(50);
        delay.delay_ms(2_000u32);

        rprintln!("Short brake for 500 ms");
        front_left_motor.brake();
        back_left_motor.brake();
        front_right_motor.brake();
        back_right_motor.brake();
        delay.delay_ms(500u32);

        rprintln!("Reverse 30 % for 2 s");
        front_left_motor.set_polarity(motor_driver::Polarity::Reverse);
        back_left_motor.set_polarity(motor_driver::Polarity::Reverse);
        front_right_motor.set_polarity(motor_driver::Polarity::Forward);
        back_right_motor.set_polarity(motor_driver::Polarity::Forward);
        front_left_motor.drive(30);
        back_left_motor.drive(30);
        front_right_motor.drive(30);
        back_right_motor.drive(30);
        delay.delay_ms(2_000u32);

        rprintln!("Short brake for 500 ms");
        front_left_motor.brake();
        back_left_motor.brake();
        front_right_motor.brake();
        back_right_motor.brake();
        delay.delay_ms(500u32);

        let mut progression = [(0.0, 0.0); 50];

        front_left_motor.set_polarity(motor_driver::Polarity::Forward);
        back_left_motor.set_polarity(motor_driver::Polarity::Forward);
        // Reverse the right motors to make the robot drive forward (installation is mirrored)
        front_right_motor.set_polarity(motor_driver::Polarity::Reverse);
        back_right_motor.set_polarity(motor_driver::Polarity::Reverse);

        rprintln!("Slowly accelerate forward to 100 % over 5 s");
        front_left_motor.smooth_drive(100.0, 5.0, &mut progression, &mut delay);
        back_left_motor.smooth_drive(100.0, 5.0, &mut progression, &mut delay);
        front_right_motor.smooth_drive(100.0, 5.0, &mut progression, &mut delay);
        back_right_motor.smooth_drive(100.0, 5.0, &mut progression, &mut delay);
        rprintln!("Slowly decelerate to stop over 5 s");
        front_left_motor.smooth_drive(0.0, 5.0, &mut progression, &mut delay);
        back_left_motor.smooth_drive(0.0, 5.0, &mut progression, &mut delay);
        front_right_motor.smooth_drive(0.0, 5.0, &mut progression, &mut delay);
        back_right_motor.smooth_drive(0.0, 5.0, &mut progression, &mut delay);

        delay.delay_ms(500u32);
    }
}
