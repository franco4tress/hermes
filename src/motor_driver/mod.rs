// ─────────────────────────────────────────────────────────────────────────────
// TB6612FNG single-channel abstraction
// ─────────────────────────────────────────────────────────────────────────────

/// Controls one motor channel of a TB6612FNG.
///
/// Truth table (STBY must be high externally):
///   AIN1 H, AIN2 L, PWM = duty  →  forward
///   AIN1 L, AIN2 H, PWM = duty  →  reverse
///   AIN1 H, AIN2 H              →  short brake
///   AIN1 L, AIN2 L, PWM = 0     →  coast / stop
use embedded_hal::{PwmPin, digital::v2::OutputPin};

pub enum Polarity {
    /// Motor spins forward (AIN1 high, AIN2 low)
    Forward,
    /// Motor spins in reverse (AIN1 low, AIN2 high)
    Reverse,
    /// Short brake (AIN1 high, AIN2 high)
    Brake,
    /// Coast/stop (AIN1 low, AIN2 low, PWM=0)
    Coast,
}

pub struct Motor<AIN1, AIN2, PWM> {
    ain1: AIN1,
    ain2: AIN2,
    pwm: PWM,
    max_duty: u16,
    polarity: Polarity,
}

impl<AIN1, AIN2, PWM> Motor<AIN1, AIN2, PWM>
where
    AIN1: OutputPin,
    AIN2: OutputPin,
    PWM: PwmPin<Duty = u16>,
{
    /// Create a new motor driver and enable PWM output.
    pub fn new(ain1: AIN1, ain2: AIN2, mut pwm: PWM, polarity: Polarity) -> Self {
        let max_duty = pwm.get_max_duty();
        pwm.enable();
        Motor {
            ain1,
            ain2,
            pwm,
            max_duty,
            polarity,
        }
    }

    /// Change direction or brake/coast mode.
    pub fn set_polarity(&mut self, polarity: Polarity) {
        match polarity {
            Polarity::Forward => {
                let _ = self.ain1.set_high();
                let _ = self.ain2.set_low();
            }
            Polarity::Reverse => {
                let _ = self.ain1.set_low();
                let _ = self.ain2.set_high();
            }
            Polarity::Brake => {
                let _ = self.ain1.set_high();
                let _ = self.ain2.set_high();
            }
            Polarity::Coast => {
                let _ = self.ain1.set_low();
                let _ = self.ain2.set_low();
            }
        }
        self.polarity = polarity;
    }
    
    /// Set speed (0-100%) in current direction or mode.
    /// Values above 100 are clamped.
    pub fn drive(&mut self, speed_pct: u8) {
        let duty = (self.max_duty as u32 * speed_pct.min(100) as u32 / 100) as u16;
        self.pwm.set_duty(duty);
    }

    pub fn forward(&mut self, speed_pct: u8) {
        self.set_polarity(Polarity::Forward);
        self.drive(speed_pct);
    }

    pub fn reverse(&mut self, speed_pct: u8) {
        self.set_polarity(Polarity::Reverse);
        self.drive(speed_pct);
    }

    /// Short-circuit brake: resist spinning, stop quickly.
    pub fn brake(&mut self) {
        self.set_polarity(Polarity::Brake);
    }

    /// Coast: outputs both low, spins freely (PWM=0).
    #[allow(dead_code)]
    pub fn coast(&mut self) {
        self.pwm.set_duty(0);
        self.set_polarity(Polarity::Coast);
    }
}
