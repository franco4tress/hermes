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
use embedded_hal::{PwmPin, digital::v2::OutputPin};

pub struct Motor<AIN1, AIN2, PWM> {
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
    pub fn new(ain1: AIN1, ain2: AIN2, mut pwm: PWM) -> Self {
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
    pub fn forward(&mut self, speed_pct: u8) {
        let duty = (self.max_duty as u32 * speed_pct.min(100) as u32 / 100) as u16;
        let _ = self.ain1.set_high();
        let _ = self.ain2.set_low();
        self.pwm.set_duty(duty);
    }

    /// Drive in reverse at `speed_pct` (0–100 %).
    pub fn reverse(&mut self, speed_pct: u8) {
        let duty = (self.max_duty as u32 * speed_pct.min(100) as u32 / 100) as u16;
        let _ = self.ain1.set_low();
        let _ = self.ain2.set_high();
        self.pwm.set_duty(duty);
    }

    /// Short-circuit brake — both outputs pulled low through the H-bridge.
    pub fn brake(&mut self) {
        let _ = self.ain1.set_high();
        let _ = self.ain2.set_high();
    }

    /// Coast — outputs go high-impedance, motor spins down on its own.
    #[allow(dead_code)]
    pub fn coast(&mut self) {
        self.pwm.set_duty(0);
        let _ = self.ain1.set_low();
        let _ = self.ain2.set_low();
    }
}
