#[derive(Debug)]
pub struct PidController {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub integral: f64,
    pub prev_error: f64,
}

impl PidController {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            prev_error: 0.0,
        }
    }

    pub fn update(&mut self, error: f64, dt: f64) -> f64 {
        self.integral += error * dt;
        let derivative = (error - self.prev_error) / dt;
        self.prev_error = error;
        
        (self.kp * error) + (self.ki * self.integral) + (self.kd * derivative)
    }
}

pub struct AlgorithmicTreasury {
    pub target_inflation: f64,
    pub current_emission_rate: f64,
    pub current_base_fee: f64,
    pub emission_pid: PidController,
    pub fee_pid: PidController,
}

impl AlgorithmicTreasury {
    pub fn new(target_inflation: f64) -> Self {
        Self {
            target_inflation,
            current_emission_rate: 100.0,
            current_base_fee: 0.01,
            emission_pid: PidController::new(0.5, 0.1, 0.05),
            fee_pid: PidController::new(0.3, 0.05, 0.01),
        }
    }

    pub fn tick(&mut self, actual_inflation: f64, dt: f64) {
        let inflation_error = self.target_inflation - actual_inflation;
        
        // If inflation is too high (negative error), emission decreases
        let emission_adjustment = self.emission_pid.update(inflation_error, dt);
        self.current_emission_rate = (self.current_emission_rate + emission_adjustment).max(0.0);
        
        // If inflation is too high, base fee increases (burning more tokens)
        let fee_error = actual_inflation - self.target_inflation;
        let fee_adjustment = self.fee_pid.update(fee_error, dt);
        self.current_base_fee = (self.current_base_fee + fee_adjustment).max(0.001);
    }
}