pub struct TreasuryLoopConfig {
    pub target_supply: f64,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub tick_interval_ms: u64,
    pub max_mint_per_tick: f64,
}

pub struct TreasuryLoopState {
    pub last_error: f64,
    pub integral: f64,
    pub last_tick_ms: u64,
    pub ticks_run: u64,
    pub total_minted: f64,
    pub total_burned: f64,
}

pub struct TreasuryLoop {
    pub config: TreasuryLoopConfig,
    pub state: TreasuryLoopState,
}

impl TreasuryLoop {
    pub fn new(config: TreasuryLoopConfig) -> Self {
        Self {
            state: TreasuryLoopState {
                last_error: 0.0,
                integral: 0.0,
                last_tick_ms: 0,
                ticks_run: 0,
                total_minted: 0.0,
                total_burned: 0.0,
            },
            config,
        }
    }

    /// PID controller tick. Returns the mint/burn amount (positive = mint, negative = burn).
    pub fn tick(&mut self, current_supply: f64, current_time_ms: u64) -> f64 {
        let dt = if self.state.last_tick_ms == 0 {
            // First tick: use configured interval as dt
            self.config.tick_interval_ms as f64 / 1000.0
        } else {
            (current_time_ms.saturating_sub(self.state.last_tick_ms)) as f64 / 1000.0
        };

        // Avoid division by zero for derivative
        let dt = if dt <= 0.0 {
            self.config.tick_interval_ms as f64 / 1000.0
        } else {
            dt
        };

        let error = self.config.target_supply - current_supply;
        self.state.integral += error * dt;
        let derivative = (error - self.state.last_error) / dt;

        let output = self.config.kp * error
            + self.config.ki * self.state.integral
            + self.config.kd * derivative;

        // Clamp to max mint/burn per tick
        let clamped = output.clamp(
            -self.config.max_mint_per_tick,
            self.config.max_mint_per_tick,
        );

        // Update state
        self.state.last_error = error;
        self.state.last_tick_ms = current_time_ms;
        self.state.ticks_run += 1;

        if clamped > 0.0 {
            self.state.total_minted += clamped;
        } else {
            self.state.total_burned += clamped.abs();
        }

        clamped
    }

    /// Resets integral and derivative (last_error) terms.
    pub fn reset(&mut self) {
        self.state.integral = 0.0;
        self.state.last_error = 0.0;
    }

    /// Returns true if the absolute last error is within the given tolerance.
    pub fn is_stable(&self, tolerance: f64) -> bool {
        self.state.last_error.abs() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TreasuryLoopConfig {
        TreasuryLoopConfig {
            target_supply: 1_000_000.0,
            kp: 0.1,
            ki: 0.01,
            kd: 0.05,
            tick_interval_ms: 1000,
            max_mint_per_tick: 10_000.0,
        }
    }

    #[test]
    fn test_pid_mints_when_below_target() {
        let mut lp = TreasuryLoop::new(default_config());
        // Supply is below target → should mint (positive output)
        let output = lp.tick(900_000.0, 1000);
        assert!(output > 0.0, "Expected mint, got {output}");
    }

    #[test]
    fn test_pid_burns_when_above_target() {
        let mut lp = TreasuryLoop::new(default_config());
        let output = lp.tick(1_100_000.0, 1000);
        assert!(output < 0.0, "Expected burn, got {output}");
    }

    #[test]
    fn test_output_clamped() {
        let mut lp = TreasuryLoop::new(default_config());
        let output = lp.tick(0.0, 1000); // max possible error
        assert!(output.abs() <= 10_000.0);
    }

    #[test]
    fn test_is_stable() {
        let mut lp = TreasuryLoop::new(default_config());
        lp.tick(1_000_000.0, 1000); // supply == target → error = 0
        assert!(lp.is_stable(1.0));
    }

    #[test]
    fn test_reset_clears_integral() {
        let mut lp = TreasuryLoop::new(default_config());
        lp.tick(500_000.0, 1000);
        lp.reset();
        assert_eq!(lp.state.integral, 0.0);
        assert_eq!(lp.state.last_error, 0.0);
    }
}
