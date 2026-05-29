import math
from scipy.stats import norm

def calculate_cpk(mu, sigma, usl):
    return (usl - mu) / (3 * sigma)

def probability_of_exhaustion(mu, sigma, usl):
    z = (usl - mu) / sigma
    return 1 - norm.cdf(z)

# PortAllocator Configuration from crates/clnrm-core/src/telemetry/live_check/port_allocator.rs
primary_range = (4317, 4327)  # 11 ports
fallback_range = (5317, 5327) # 11 ports
extended_range = (6317, 7337) # 1021 ports

total_capacity = (primary_range[1] - primary_range[0] + 1) + \
                 (fallback_range[1] - fallback_range[0] + 1) + \
                 (extended_range[1] - extended_range[0] + 1)

print(f"Total Port Capacity: {total_capacity}")

# Scenario: 1,000 allocations (High-density parallel CI)
mu = 1000
# Assuming Poisson-like demand for concurrent jobs
sigma = math.sqrt(mu) 

print(f"Mean Demand (mu): {mu}")
print(f"Standard Deviation (sigma): {sigma:.2f}")

# Overall Capability
cpk_total = calculate_cpk(mu, sigma, total_capacity)
prob_exhaustion = probability_of_exhaustion(mu, sigma, total_capacity)

print(f"\n--- Overall Process Capability ---")
print(f"Cpk: {cpk_total:.4f}")
print(f"Probability of Port Exhaustion: {prob_exhaustion:.4%}")
print(f"Sigma Level: {cpk_total * 3:.2f}")

# Primary Range Capability
cpk_primary = calculate_cpk(mu, sigma, 11)
print(f"\n--- Primary Range Capability (11 ports) ---")
print(f"Cpk: {cpk_primary:.4f}")
print(f"Status: {'INCAPABLE' if cpk_primary < 1.33 else 'CAPABLE'}")

# Fallback Range Capability
cpk_fallback = calculate_cpk(mu, sigma, 22) # Primary + Fallback
print(f"\n--- Primary + Fallback Range Capability (22 ports) ---")
print(f"Cpk: {cpk_fallback:.4f}")
print(f"Status: {'INCAPABLE' if cpk_fallback < 1.33 else 'CAPABLE'}")

# Proof of Incapability
print(f"\n--- Proof of Incapability ---")
print(f"1. A process is considered 'Capable' if Cpk >= 1.33.")
print(f"2. Current Total Cpk is {cpk_total:.4f}, which is < 1.33.")
print(f"3. Failure rate (Exhaustion) is {prob_exhaustion:.2%}, which exceeds industrial standards (usually < 0.27% for 3rd sigma).")
print(f"4. The Primary range is undersized by a factor of {mu/11:.1f}x relative to expected mean demand.")

required_capacity = mu + (1.33 * 3 * sigma)
print(f"5. Required capacity for Cpk=1.33 at mu=1000 is ~{math.ceil(required_capacity)} ports.")
print(f"6. Deficit: {math.ceil(required_capacity) - total_capacity} ports.")
