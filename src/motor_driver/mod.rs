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
    throttle: f32,
}

impl<AIN1, AIN2, PWM> Motor<AIN1, AIN2, PWM>
where
    AIN1: OutputPin,
    AIN2: OutputPin,
    PWM: PwmPin<Duty = u16>,
{
    /// Create a new motor driver and enable PWM output.
    pub fn new(ain1: AIN1, ain2: AIN2, mut pwm: PWM, polarity: Polarity, throttle: f32) -> Self {
        let max_duty = pwm.get_max_duty();
        pwm.enable();
        Motor {
            ain1,
            ain2,
            pwm,
            max_duty,
            polarity,
            throttle,
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
    pub fn drive(&mut self, speed_pct: impl Into<u8>) {
        let speed_pct = speed_pct.into();
        let duty = (self.max_duty as u32 * speed_pct.min(100) as u32 / 100) as u16;
        self.pwm.set_duty(duty);
        self.throttle = speed_pct as f32;
    }

    /// Short-circuit brake: resist spinning, stop quickly.
    pub fn brake(&mut self) {
        self.set_polarity(Polarity::Brake);
        self.throttle = 0.0;
    }

    /// Coast: outputs both low, spins freely (PWM=0).
    #[allow(dead_code)]
    pub fn coast(&mut self) {
        self.pwm.set_duty(0);
        self.set_polarity(Polarity::Coast);
        // NOTE: because coasting allows free spinning, throttle cannot be meaningfully tracked in this mode.
        // Throttle is 0 for simplicity, but rotary encoders or other feedback would be needed to track actual speed.
        self.throttle = 0.0;
    }

    /// Smoothly drive to a target throttle, filling the provided progression buffer.
    /// The user provides the buffer, and the function fills it and drives the motor.
    ///
    /// NOTE: The blocking delay logic is for demonstration only and should be removed for RTOS or async use.
    pub fn smooth_drive<D>(&mut self, target_pct: f32, total_time: f32, progression: &mut [(f32, f32)], delay: &mut D)
    where
        D: embedded_hal::blocking::delay::DelayMs<u32>,
    {
        self.generate_throttle_progression(target_pct, total_time, progression);
        let mut last_time = 0.0;
        for &(time, pct) in progression.iter() {
            // --- BEGIN: REMOVE THIS BLOCK FOR NON-BLOCKING/RTOS USE ---
            let ms = ((time - last_time).max(0.0) * 1000.0) as u32;
            if ms > 0 {
                delay.delay_ms(ms);
            }
            last_time = time;
            // --- END: REMOVE THIS BLOCK FOR NON-BLOCKING/RTOS USE ---
            self.drive(pct as u8);
        }
    }

    // Generates a throttle progression from self.throttle to target_pct over total_time,
    // filling the out buffer with (time, throttle) samples.
    fn generate_throttle_progression(
        &self,
        target_pct: f32,
        total_time: f32,
        out: &mut [(f32, f32)],
    ) {
        let steps = out.len();
        if steps == 0 || total_time < 0.0 || self.throttle < 0.0 || target_pct < 0.0 {
            return;
        }
        if total_time == 0.0 {
            out[0] = (0.0, target_pct);
            return;
        }
        if steps == 1 {
            out[0] = (total_time, target_pct);
            return;
        }
        let dt = total_time / (steps as f32 - 1.0);
        let curve_fn = Self::get_curve(self.throttle, target_pct, total_time);
        for (i, slot) in out.iter_mut().enumerate() {
            let t = dt * (i as f32);
            *slot = (t, curve_fn(t));
        }
    }

    // Returns a closure that computes the throttle percentage at a given time.
    fn get_curve(init_pct: f32, target_pct: f32, total_time: f32) -> impl Fn(f32) -> f32 {
        move |time| {
            let norm_time = Self::clamp_and_norm(time, total_time);
            init_pct + (target_pct - init_pct) * norm_time * norm_time * norm_time * (norm_time * (6.0 * norm_time - 15.0) + 10.0)
        }
    }

    // Clamps value to [0,1] and normalizes w.r.t. upper.
    fn clamp_and_norm(value: f32, upper: f32) -> f32 {
        match value {
            v if v < 0.0 => 0.0,
            v if v > upper => 1.0,
            v => v / upper,
        }
    }
}
