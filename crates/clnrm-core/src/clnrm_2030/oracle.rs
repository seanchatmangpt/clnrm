use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OracleDataPoint {
    pub value: f64,
    pub timestamp: u64,
    pub provider: String,
    pub stake: u64,
}

pub struct DecentralizedOracle {
    streams: HashMap<String, Vec<OracleDataPoint>>,
}

impl Default for DecentralizedOracle {
    fn default() -> Self {
        Self::new()
    }
}

impl DecentralizedOracle {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    pub fn submit_data(&mut self, stream_id: &str, point: OracleDataPoint) {
        self.streams.entry(stream_id.to_string()).or_insert_with(Vec::new).push(point);
    }

    pub fn aggregate(&self, stream_id: &str) -> Option<f64> {
        let points = self.streams.get(stream_id)?;
        if points.is_empty() {
            return None;
        }

        // We use a staked-weighted median to discount outliers
        let mut weighted_points: Vec<(f64, u64)> = points.iter().map(|p| (p.value, p.stake)).collect();
        weighted_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let total_stake: u64 = weighted_points.iter().map(|(_, s)| s).sum();
        let target_weight = total_stake / 2;

        let mut cumulative_weight = 0;
        for (value, stake) in weighted_points {
            cumulative_weight += stake;
            if cumulative_weight >= target_weight {
                return Some(value);
            }
        }
        
        Some(points[0].value) // Fallback
    }
}