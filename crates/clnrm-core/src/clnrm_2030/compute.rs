use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

#[derive(Debug, Clone)]
pub struct ComputeContract {
    pub provider: NodeId,
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub bandwidth_mbps: u32,
    pub price_per_second: u64,
    pub available: bool,
}

pub struct ComputeMarket {
    inventory: HashMap<NodeId, ComputeContract>,
    base_demand_multiplier: f64,
}

impl Default for ComputeMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeMarket {
    pub fn new() -> Self {
        Self {
            inventory: HashMap::new(),
            base_demand_multiplier: 1.0,
        }
    }

    pub fn list_compute(&mut self, contract: ComputeContract) {
        self.inventory.insert(contract.provider.clone(), contract);
    }

    pub fn update_network_utilization(&mut self, utilization_percentage: f64) {
        // Dynamic pricing model based on DAO utilization
        if utilization_percentage > 0.8 {
            self.base_demand_multiplier = 1.0 + (utilization_percentage - 0.8) * 5.0;
        // Surge pricing
        } else {
            self.base_demand_multiplier = 1.0;
        }
    }

    pub fn get_spot_price(&self, provider: &NodeId) -> Option<u64> {
        let contract = self.inventory.get(provider)?;
        Some((contract.price_per_second as f64 * self.base_demand_multiplier) as u64)
    }

    pub fn lease(&mut self, provider: &NodeId) -> Result<ComputeContract, &'static str> {
        let contract = self
            .inventory
            .get_mut(provider)
            .ok_or("Provider not found")?;
        if !contract.available {
            return Err("Compute resources currently leased");
        }
        contract.available = false;
        Ok(contract.clone())
    }
}
