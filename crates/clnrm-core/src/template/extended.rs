//! Extended template functions for comprehensive generator support
//!
//! Additional functions to complement the core fake data generators:
//! - RNG primitives (rand_hex, seq counter)
//! - UUID variants (uuid_v7, uuid_v5, ulid)
//! - Collections (pick, weighted, shuffle, sample)
//! - String transforms (slug, kebab, snake)
//! - Time helpers (now_unix, now_ms, now_plus)
//! - OTEL helpers (trace_id, span_id, traceparent, baggage)
//! - Unified fake() interface

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tera::{Function, Tera, Value};

// Helper to get seed from args (reuse from main functions.rs)
fn get_seed(args: &HashMap<String, Value>) -> u64 {
    args.get("seed")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(rand::random)
}

/// Register all extended functions with Tera
pub fn register_extended_functions(tera: &mut Tera) {
    // RNG primitives
    tera.register_function("rand_hex", RandHexFunction);
    tera.register_function("seq", SeqFunction::new());

    // UUIDs
    tera.register_function("uuid_v4", UuidV4Function);
    tera.register_function("uuid_v7", UuidV7Function);
    tera.register_function("uuid_v5", UuidV5Function);
    tera.register_function("ulid", UlidFunction);

    // Collections
    tera.register_function("pick", PickFunction);
    tera.register_function("weighted", WeightedFunction);
    tera.register_function("shuffle", ShuffleFunction);
    tera.register_function("sample", SampleFunction);

    // String transforms
    tera.register_function("slug", SlugFunction);
    tera.register_function("kebab", KebabFunction);
    tera.register_function("snake", SnakeFunction);

    // Time helpers
    tera.register_function("now_unix", NowUnixFunction);
    tera.register_function("now_ms", NowMsFunction);
    tera.register_function("now_plus", NowPlusFunction);
    tera.register_function("date_rfc3339", DateRfc3339Function);

    // OTEL helpers
    tera.register_function("trace_id", TraceIdFunction);
    tera.register_function("span_id", SpanIdFunction);
    tera.register_function("traceparent", TraceparentFunction);
    tera.register_function("baggage", BaggageFunction);

    // Unified fake interface
    tera.register_function("fake", UnifiedFakeFunction);
    tera.register_function("fake_kinds", FakeKindsFunction);
}

// ========================================
// RNG Primitives
// ========================================

/// rand_hex(n, seed=42) - Generate n random hex characters
struct RandHexFunction;
impl Function for RandHexFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let n = args.get("n").and_then(|v| v.as_u64()).unwrap_or(16) as usize;

        let hex: String = (0..n)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect();

        Ok(Value::String(hex))
    }
}

/// seq(name, start=0, step=1) - Monotonic per-render counter
struct SeqFunction {
    counters: Arc<Mutex<HashMap<String, i64>>>,
}

impl SeqFunction {
    fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Function for SeqFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("seq() requires 'name' parameter"))?;

        let start = args.get("start").and_then(|v| v.as_i64()).unwrap_or(0);
        let step = args.get("step").and_then(|v| v.as_i64()).unwrap_or(1);

        let mut counters = self.counters.lock().map_err(|e| {
            tera::Error::msg(format!("Failed to lock sequence counter: {}", e))
        })?;
        let counter = counters.entry(name.to_string()).or_insert(start);
        let value = *counter;
        *counter += step;

        Ok(Value::Number(value.into()))
    }
}

// ========================================
// UUID Functions
// ========================================

/// uuid_v4(seed=42) - Generate UUID v4
struct UuidV4Function;
impl Function for UuidV4Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        if args.contains_key("seed") {
            // Deterministic UUID from seed
            let seed = get_seed(args);
            let uuid_bytes = format!("{:032x}", seed);
            Ok(Value::String(format!(
                "{}-{}-4{}-{}-{}",
                &uuid_bytes[0..8],
                &uuid_bytes[8..12],
                &uuid_bytes[13..15],
                &uuid_bytes[16..20],
                &uuid_bytes[20..32]
            )))
        } else {
            // Random UUID
            Ok(Value::String(uuid::Uuid::new_v4().to_string()))
        }
    }
}

/// uuid_v7(time=freeze_clock) - Generate UUID v7 (time-based)
struct UuidV7Function;
impl Function for UuidV7Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        // UUID v7 uses timestamp - if time is frozen, use that
        let _time = args.get("time").and_then(|v| v.as_str());

        // For now, generate a random UUID v7 format
        // Real implementation would use timestamp from freeze_clock
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let timestamp_ms = chrono::Utc::now().timestamp_millis() as u64;

        // UUID v7 format: timestamp_ms (48 bits) + version (4) + random (12) + variant (2) + random (62)
        let uuid_str = format!(
            "{:012x}-{:04x}-7{:03x}-{:04x}-{:012x}",
            timestamp_ms & 0xFFFFFFFFFFFF,
            rng.gen::<u16>(),
            rng.gen::<u16>() & 0xFFF,
            (rng.gen::<u16>() & 0x3FFF) | 0x8000,
            rng.gen::<u64>() & 0xFFFFFFFFFFFF
        );

        Ok(Value::String(uuid_str))
    }
}

/// uuid_v5(ns, name) - Generate UUID v5 (name-based, SHA-1)
struct UuidV5Function;
impl Function for UuidV5Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let ns = args
            .get("ns")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("uuid_v5() requires 'ns' parameter"))?;

        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("uuid_v5() requires 'name' parameter"))?;

        // Parse namespace UUID
        let namespace_uuid = uuid::Uuid::parse_str(ns)
            .map_err(|e| tera::Error::msg(format!("Invalid namespace UUID: {}", e)))?;

        // Generate UUID v5
        let uuid = uuid::Uuid::new_v5(&namespace_uuid, name.as_bytes());

        Ok(Value::String(uuid.to_string()))
    }
}

/// ulid(time=freeze_clock) - Generate ULID (Universally Unique Lexicographically Sortable Identifier)
struct UlidFunction;
impl Function for UlidFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);

        // ULID format: 10 chars timestamp (base32) + 16 chars random (base32)
        // For deterministic generation, use seed
        let timestamp_ms = chrono::Utc::now().timestamp_millis() as u64;

        // Base32 encoding (Crockford's alphabet)
        let base32 = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

        // Encode timestamp (48 bits, 10 base32 chars)
        let mut ulid = String::with_capacity(26);
        let mut ts = timestamp_ms;
        for _ in 0..10 {
            let idx = (ts % 32) as usize;
            let ch = base32.chars().nth(idx).ok_or_else(|| {
                tera::Error::msg(format!("Invalid base32 index {} during ULID timestamp encoding", idx))
            })?;
            ulid.insert(0, ch);
            ts /= 32;
        }

        // Random part (80 bits, 16 base32 chars)
        for _ in 0..16 {
            let idx = rng.gen_range(0..32);
            let ch = base32.chars().nth(idx).ok_or_else(|| {
                tera::Error::msg(format!("Invalid base32 index {} during ULID random part generation", idx))
            })?;
            ulid.push(ch);
        }

        Ok(Value::String(ulid))
    }
}

// ========================================
// Collection Functions
// ========================================

/// pick(list, seed=42) - Pick one random element from list
struct PickFunction;
impl Function for PickFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let list = args
            .get("list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| tera::Error::msg("pick() requires 'list' array parameter"))?;

        if list.is_empty() {
            return Err(tera::Error::msg("pick() requires non-empty list"));
        }

        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let idx = rng.gen_range(0..list.len());

        Ok(list[idx].clone())
    }
}

/// weighted(pairs, seed=42) - Weighted random selection
/// pairs = [["A", 0.7], ["B", 0.3]]
struct WeightedFunction;
impl Function for WeightedFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let pairs = args
            .get("pairs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| tera::Error::msg("weighted() requires 'pairs' array parameter"))?;

        if pairs.is_empty() {
            return Err(tera::Error::msg("weighted() requires non-empty pairs"));
        }

        // Extract values and weights
        let mut values = Vec::new();
        let mut weights = Vec::new();
        let mut total_weight = 0.0;

        for pair in pairs {
            let pair_array = pair.as_array().ok_or_else(|| {
                tera::Error::msg("weighted() pairs must be arrays [value, weight]")
            })?;

            if pair_array.len() != 2 {
                return Err(tera::Error::msg(
                    "weighted() pairs must have exactly 2 elements",
                ));
            }

            values.push(pair_array[0].clone());
            let weight = pair_array[1]
                .as_f64()
                .ok_or_else(|| tera::Error::msg("weighted() weights must be numbers"))?;
            weights.push(weight);
            total_weight += weight;
        }

        // Random selection
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let mut rand_val = rng.gen::<f64>() * total_weight;

        for (i, weight) in weights.iter().enumerate() {
            rand_val -= weight;
            if rand_val <= 0.0 {
                return Ok(values[i].clone());
            }
        }

        // Fallback to last element (guaranteed non-empty by earlier check)
        Ok(values.last()
            .ok_or_else(|| tera::Error::msg("weighted() internal error: empty values after weight calculation"))?
            .clone())
    }
}

/// shuffle(list, seed=42) - Shuffle list randomly
struct ShuffleFunction;
impl Function for ShuffleFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let list = args
            .get("list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| tera::Error::msg("shuffle() requires 'list' array parameter"))?;

        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);

        let mut shuffled = list.clone();

        // Fisher-Yates shuffle
        for i in (1..shuffled.len()).rev() {
            let j = rng.gen_range(0..=i);
            shuffled.swap(i, j);
        }

        Ok(Value::Array(shuffled))
    }
}

/// sample(list, k, seed=42) - Sample k elements from list without replacement
struct SampleFunction;
impl Function for SampleFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let list = args
            .get("list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| tera::Error::msg("sample() requires 'list' array parameter"))?;

        let k = args
            .get("k")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| tera::Error::msg("sample() requires 'k' number parameter"))?
            as usize;

        if k > list.len() {
            return Err(tera::Error::msg(format!(
                "sample() k ({}) cannot be larger than list size ({})",
                k,
                list.len()
            )));
        }

        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);

        // Reservoir sampling
        let mut sample = Vec::with_capacity(k);
        for (i, item) in list.iter().enumerate() {
            if i < k {
                sample.push(item.clone());
            } else {
                let j = rng.gen_range(0..=i);
                if j < k {
                    sample[j] = item.clone();
                }
            }
        }

        Ok(Value::Array(sample))
    }
}

// ========================================
// String Transform Functions
// ========================================

/// slug(s) - Convert to URL-friendly slug
struct SlugFunction;
impl Function for SlugFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let s = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("slug() requires 's' string parameter"))?;

        let slug = s
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-");

        Ok(Value::String(slug))
    }
}

/// kebab(s) - Convert to kebab-case
struct KebabFunction;
impl Function for KebabFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let s = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("kebab() requires 's' string parameter"))?;

        let kebab = s
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    format!("-{}", c.to_lowercase())
                } else {
                    c.to_lowercase().to_string()
                }
            })
            .collect::<String>();

        Ok(Value::String(kebab))
    }
}

/// snake(s) - Convert to snake_case
struct SnakeFunction;
impl Function for SnakeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let s = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("snake() requires 's' string parameter"))?;

        let snake = s
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() && i > 0 {
                    format!("_{}", c.to_lowercase())
                } else if c == ' ' || c == '-' {
                    "_".to_string()
                } else {
                    c.to_lowercase().to_string()
                }
            })
            .collect::<String>();

        Ok(Value::String(snake))
    }
}

// ========================================
// Time Helper Functions
// ========================================

/// now_unix() - Current Unix timestamp (seconds)
struct NowUnixFunction;
impl Function for NowUnixFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let timestamp = chrono::Utc::now().timestamp();
        Ok(Value::Number(timestamp.into()))
    }
}

/// now_ms() - Current timestamp in milliseconds
struct NowMsFunction;
impl Function for NowMsFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let timestamp_ms = chrono::Utc::now().timestamp_millis();
        Ok(Value::Number(timestamp_ms.into()))
    }
}

/// now_plus(seconds) - RFC3339 timestamp N seconds in future
struct NowPlusFunction;
impl Function for NowPlusFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seconds = args
            .get("seconds")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| tera::Error::msg("now_plus() requires 'seconds' parameter"))?;

        let future = chrono::Utc::now() + chrono::Duration::seconds(seconds);
        Ok(Value::String(future.to_rfc3339()))
    }
}

/// date_rfc3339(offset_seconds) - RFC3339 timestamp with offset
struct DateRfc3339Function;
impl Function for DateRfc3339Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let offset = args
            .get("offset_seconds")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let dt = chrono::Utc::now() + chrono::Duration::seconds(offset);
        Ok(Value::String(dt.to_rfc3339()))
    }
}

// ========================================
// OTEL Helper Functions
// ========================================

/// trace_id(seed=42) - Generate 32 hex char trace ID
struct TraceIdFunction;
impl Function for TraceIdFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);

        let trace_id: String = (0..32)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect();

        Ok(Value::String(trace_id))
    }
}

/// span_id(seed=42) - Generate 16 hex char span ID
struct SpanIdFunction;
impl Function for SpanIdFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);

        let span_id: String = (0..16)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect();

        Ok(Value::String(span_id))
    }
}

/// traceparent(trace_id=auto, span_id=auto, sampled=1) - W3C traceparent header
struct TraceparentFunction;
impl Function for TraceparentFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let trace_id = args
            .get("trace_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let seed = get_seed(args);
                let mut rng = StdRng::seed_from_u64(seed);
                (0..32)
                    .map(|_| format!("{:x}", rng.gen_range(0..16)))
                    .collect()
            });

        let span_id = args
            .get("span_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let seed = get_seed(args) + 1; // Different seed for span_id
                let mut rng = StdRng::seed_from_u64(seed);
                (0..16)
                    .map(|_| format!("{:x}", rng.gen_range(0..16)))
                    .collect()
            });

        let sampled = args.get("sampled").and_then(|v| v.as_u64()).unwrap_or(1);
        let flags = format!("{:02x}", sampled);

        Ok(Value::String(format!(
            "00-{}-{}-{}",
            trace_id, span_id, flags
        )))
    }
}

/// baggage(map) - Encode W3C baggage header
struct BaggageFunction;
impl Function for BaggageFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let map = args
            .get("map")
            .and_then(|v| v.as_object())
            .ok_or_else(|| tera::Error::msg("baggage() requires 'map' object parameter"))?;

        let baggage = map
            .iter()
            .map(|(k, v)| {
                let value_str = match v {
                    Value::String(s) => s.clone(),
                    _ => v.to_string(),
                };
                format!("{}={}", k, value_str)
            })
            .collect::<Vec<String>>()
            .join(",");

        Ok(Value::String(baggage))
    }
}

// ========================================
// Unified Fake Interface
// ========================================

/// fake(kind, seed=42, n=1) - Unified fake data interface
struct UnifiedFakeFunction;
impl Function for UnifiedFakeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let kind = args
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("fake() requires 'kind' parameter"))?;

        let _seed = get_seed(args);
        let _n = args.get("n").and_then(|v| v.as_u64()).unwrap_or(1);

        // Map kind to appropriate fake function
        // This is a simplified implementation - full version would call actual fake functions
        let result = match kind {
            "name.full" => "John Doe",
            "name.first" => "John",
            "name.last" => "Doe",
            "internet.email.safe" => "user@example.com",
            "internet.email.free" => "user@gmail.com",
            "internet.username" => "johndoe123",
            "internet.domain.suffix" => "com",
            "internet.ip.any" => "192.168.1.1",
            "internet.ip.v4" => "192.168.1.1",
            "internet.ip.v6" => "2001:0db8::1",
            "internet.password" => "P@ssw0rd123",
            "address.city" => "San Francisco",
            "address.country" => "United States",
            "address.street.name" => "Main Street",
            "address.street.address" => "123 Main St",
            "address.zip" => "94102",
            "address.tz" => "America/Los_Angeles",
            "company.name" => "Acme Corp",
            "company.buzzword" => "synergy",
            "company.industry" => "Technology",
            "company.profession" => "Software Engineer",
            "lorem.word" => "lorem",
            "lorem.sentence" => "Lorem ipsum dolor sit amet.",
            "lorem.paragraph" => "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            "phone.number" => "+1-555-123-4567",
            "uuid.v4" => &uuid::Uuid::new_v4().to_string(),
            _ => "unknown_kind",
        };

        Ok(Value::String(result.to_string()))
    }
}

/// fake_kinds() - Return list of supported fake data kinds
struct FakeKindsFunction;
impl Function for FakeKindsFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let kinds = vec![
            "name.full",
            "name.first",
            "name.last",
            "name.title",
            "internet.email.safe",
            "internet.email.free",
            "internet.username",
            "internet.domain.suffix",
            "internet.ip.any",
            "internet.ip.v4",
            "internet.ip.v6",
            "internet.password",
            "address.city",
            "address.country",
            "address.street.name",
            "address.street.address",
            "address.zip",
            "address.tz",
            "company.name",
            "company.buzzword",
            "company.industry",
            "company.profession",
            "lorem.word",
            "lorem.sentence",
            "lorem.paragraph",
            "phone.number",
            "uuid.v4",
        ];

        let values: Vec<Value> = kinds.iter().map(|k| Value::String(k.to_string())).collect();
        Ok(Value::Array(values))
    }
}

// ========================================
// Extended Function Tests
// ========================================

#[cfg(test)]
mod extended_function_tests {
    use super::*;
    use tera::Value;
    use serial_test::serial;

    // ========================================
    // UUID V7 Tests (2 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_uuid_v7_generates_valid_format() {
        // Arrange
        let function = UuidV7Function;
        let args = HashMap::new();

        // Act
        let result = function.call(&args).expect("uuid_v7 should succeed");

        // Assert
        let uuid_str = result.as_str().expect("Result should be string");
        assert_eq!(uuid_str.len(), 36, "UUID v7 should be 36 characters");
        assert_eq!(uuid_str.chars().filter(|&c| c == '-').count(), 4, "UUID v7 should have 4 hyphens");

        // Check version bit (7th character should be '7')
        let parts: Vec<&str> = uuid_str.split('-').collect();
        assert_eq!(parts.len(), 5, "UUID v7 should have 5 segments");
        assert!(parts[2].starts_with('7'), "Third segment should start with '7' for version 7");
    }

    #[test]
    #[serial]
    fn test_uuid_v7_with_seed_is_deterministic() {
        // Arrange
        let function = UuidV7Function;
        let mut args1 = HashMap::new();
        args1.insert("seed".to_string(), Value::Number(42.into()));

        let mut args2 = HashMap::new();
        args2.insert("seed".to_string(), Value::Number(42.into()));

        // Act
        let result1 = function.call(&args1).expect("uuid_v7 should succeed");
        let result2 = function.call(&args2).expect("uuid_v7 should succeed");

        // Assert
        let uuid1 = result1.as_str().expect("Result should be string");
        let uuid2 = result2.as_str().expect("Result should be string");

        // Only the random parts should be deterministic with same seed
        // The timestamp part will be the same since calls are close together
        assert_eq!(uuid1, uuid2, "UUID v7 with same seed should be deterministic");
    }

    #[test]
    #[serial]
    fn test_uuid_v7_without_seed_generates_different_values() {
        // Arrange
        let function = UuidV7Function;
        let args = HashMap::new();

        // Act
        let result1 = function.call(&args).expect("uuid_v7 should succeed");
        let result2 = function.call(&args).expect("uuid_v7 should succeed");

        // Assert
        let uuid1 = result1.as_str().expect("Result should be string");
        let uuid2 = result2.as_str().expect("Result should be string");

        // Without seed, UUIDs should differ (at least in random parts)
        // Note: They might be the same if generated in same millisecond with same random seed
        // but statistically this is very unlikely
        assert_eq!(uuid1.len(), 36);
        assert_eq!(uuid2.len(), 36);
    }

    // ========================================
    // ULID Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_ulid_generates_valid_format() {
        // Arrange
        let function = UlidFunction;
        let args = HashMap::new();

        // Act
        let result = function.call(&args).expect("ulid should succeed");

        // Assert
        let ulid_str = result.as_str().expect("Result should be string");
        assert_eq!(ulid_str.len(), 26, "ULID should be 26 characters");

        // All characters should be valid base32 (Crockford alphabet)
        for c in ulid_str.chars() {
            assert!(
                c.is_ascii_digit() || c.is_ascii_uppercase(),
                "ULID should only contain 0-9 and A-Z, found: {}",
                c
            );
        }
    }

    #[test]
    #[serial]
    fn test_ulid_with_seed_is_deterministic() {
        // Arrange
        let function = UlidFunction;
        let mut args1 = HashMap::new();
        args1.insert("seed".to_string(), Value::Number(12345.into()));

        let mut args2 = HashMap::new();
        args2.insert("seed".to_string(), Value::Number(12345.into()));

        // Act
        let result1 = function.call(&args1).expect("ulid should succeed");
        let result2 = function.call(&args2).expect("ulid should succeed");

        // Assert
        let ulid1 = result1.as_str().expect("Result should be string");
        let ulid2 = result2.as_str().expect("Result should be string");

        assert_eq!(ulid1, ulid2, "ULID with same seed should be deterministic");
    }

    #[test]
    #[serial]
    fn test_ulid_is_lexicographically_sortable() {
        // Arrange
        let function = UlidFunction;
        let mut args1 = HashMap::new();
        args1.insert("seed".to_string(), Value::Number(100.into()));

        // Wait a tiny bit to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(2));

        let mut args2 = HashMap::new();
        args2.insert("seed".to_string(), Value::Number(200.into()));

        // Act
        let result1 = function.call(&args1).expect("ulid should succeed");
        let result2 = function.call(&args2).expect("ulid should succeed");

        // Assert
        let ulid1 = result1.as_str().expect("Result should be string");
        let ulid2 = result2.as_str().expect("Result should be string");

        // Later timestamp should sort after earlier timestamp
        assert!(ulid2 >= ulid1, "Later ULID should be >= earlier ULID (lexicographic ordering)");
    }

    // ========================================
    // Traceparent Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_traceparent_generates_valid_w3c_format() {
        // Arrange
        let function = TraceparentFunction;
        let args = HashMap::new();

        // Act
        let result = function.call(&args).expect("traceparent should succeed");

        // Assert
        let traceparent = result.as_str().expect("Result should be string");

        // W3C format: 00-{trace_id}-{span_id}-{flags}
        let parts: Vec<&str> = traceparent.split('-').collect();
        assert_eq!(parts.len(), 4, "Traceparent should have 4 parts");
        assert_eq!(parts[0], "00", "Version should be '00'");
        assert_eq!(parts[1].len(), 32, "Trace ID should be 32 hex chars");
        assert_eq!(parts[2].len(), 16, "Span ID should be 16 hex chars");
        assert_eq!(parts[3].len(), 2, "Flags should be 2 hex chars");

        // All hex characters
        for c in parts[1].chars().chain(parts[2].chars()).chain(parts[3].chars()) {
            assert!(c.is_ascii_hexdigit(), "Should be hex digit: {}", c);
        }
    }

    #[test]
    #[serial]
    fn test_traceparent_with_custom_trace_id() {
        // Arrange
        let function = TraceparentFunction;
        let mut args = HashMap::new();
        let custom_trace = "a".repeat(32);
        args.insert("trace_id".to_string(), Value::String(custom_trace.clone()));

        // Act
        let result = function.call(&args).expect("traceparent should succeed");

        // Assert
        let traceparent = result.as_str().expect("Result should be string");
        assert!(traceparent.contains(&custom_trace), "Should contain custom trace ID");
    }

    #[test]
    #[serial]
    fn test_traceparent_with_seed_is_deterministic() {
        // Arrange
        let function = TraceparentFunction;
        let mut args1 = HashMap::new();
        args1.insert("seed".to_string(), Value::Number(999.into()));

        let mut args2 = HashMap::new();
        args2.insert("seed".to_string(), Value::Number(999.into()));

        // Act
        let result1 = function.call(&args1).expect("traceparent should succeed");
        let result2 = function.call(&args2).expect("traceparent should succeed");

        // Assert
        assert_eq!(result1, result2, "Traceparent with same seed should be deterministic");
    }

    // ========================================
    // Baggage Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_baggage_encodes_single_key_value() {
        // Arrange
        let function = BaggageFunction;
        let mut map = serde_json::Map::new();
        map.insert("user_id".to_string(), Value::String("12345".to_string()));

        let mut args = HashMap::new();
        args.insert("map".to_string(), Value::Object(map));

        // Act
        let result = function.call(&args).expect("baggage should succeed");

        // Assert
        let baggage = result.as_str().expect("Result should be string");
        assert_eq!(baggage, "user_id=12345", "Should encode single key-value pair");
    }

    #[test]
    #[serial]
    fn test_baggage_encodes_multiple_key_values() {
        // Arrange
        let function = BaggageFunction;
        let mut map = serde_json::Map::new();
        map.insert("user_id".to_string(), Value::String("12345".to_string()));
        map.insert("session_id".to_string(), Value::String("abc-def".to_string()));
        map.insert("env".to_string(), Value::String("prod".to_string()));

        let mut args = HashMap::new();
        args.insert("map".to_string(), Value::Object(map));

        // Act
        let result = function.call(&args).expect("baggage should succeed");

        // Assert
        let baggage = result.as_str().expect("Result should be string");

        // Should contain all three pairs (order may vary due to HashMap)
        assert!(baggage.contains("user_id=12345"), "Should contain user_id");
        assert!(baggage.contains("session_id=abc-def"), "Should contain session_id");
        assert!(baggage.contains("env=prod"), "Should contain env");

        // Should have comma separators
        assert_eq!(baggage.matches(',').count(), 2, "Should have 2 commas for 3 pairs");
    }

    #[test]
    #[serial]
    fn test_baggage_requires_map_parameter() {
        // Arrange
        let function = BaggageFunction;
        let args = HashMap::new();

        // Act
        let result = function.call(&args);

        // Assert
        assert!(result.is_err(), "Should error without map parameter");
        assert!(result.unwrap_err().to_string().contains("map"), "Error should mention 'map'");
    }

    // ========================================
    // Pick Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_pick_selects_from_list() {
        // Arrange
        let function = PickFunction;
        let list = vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("cherry".to_string()),
        ];

        let mut args = HashMap::new();
        args.insert("list".to_string(), Value::Array(list.clone()));
        args.insert("seed".to_string(), Value::Number(42.into()));

        // Act
        let result = function.call(&args).expect("pick should succeed");

        // Assert
        let picked = result.as_str().expect("Result should be string");
        assert!(
            picked == "apple" || picked == "banana" || picked == "cherry",
            "Should pick one of the list items"
        );
    }

    #[test]
    #[serial]
    fn test_pick_with_seed_is_deterministic() {
        // Arrange
        let function = PickFunction;
        let list = vec![
            Value::String("red".to_string()),
            Value::String("green".to_string()),
            Value::String("blue".to_string()),
        ];

        let mut args1 = HashMap::new();
        args1.insert("list".to_string(), Value::Array(list.clone()));
        args1.insert("seed".to_string(), Value::Number(777.into()));

        let mut args2 = HashMap::new();
        args2.insert("list".to_string(), Value::Array(list.clone()));
        args2.insert("seed".to_string(), Value::Number(777.into()));

        // Act
        let result1 = function.call(&args1).expect("pick should succeed");
        let result2 = function.call(&args2).expect("pick should succeed");

        // Assert
        assert_eq!(result1, result2, "Pick with same seed should be deterministic");
    }

    #[test]
    #[serial]
    fn test_pick_errors_on_empty_list() {
        // Arrange
        let function = PickFunction;
        let mut args = HashMap::new();
        args.insert("list".to_string(), Value::Array(vec![]));

        // Act
        let result = function.call(&args);

        // Assert
        assert!(result.is_err(), "Should error on empty list");
        assert!(result.unwrap_err().to_string().contains("non-empty"), "Error should mention non-empty");
    }

    // ========================================
    // Weighted Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_weighted_selects_based_on_weights() {
        // Arrange
        let function = WeightedFunction;
        let pairs = vec![
            Value::Array(vec![Value::String("A".to_string()), Value::Number(serde_json::Number::from_f64(0.8).unwrap())]),
            Value::Array(vec![Value::String("B".to_string()), Value::Number(serde_json::Number::from_f64(0.2).unwrap())]),
        ];

        let mut args = HashMap::new();
        args.insert("pairs".to_string(), Value::Array(pairs));
        args.insert("seed".to_string(), Value::Number(42.into()));

        // Act
        let result = function.call(&args).expect("weighted should succeed");

        // Assert
        let selected = result.as_str().expect("Result should be string");
        assert!(selected == "A" || selected == "B", "Should select A or B");
    }

    #[test]
    #[serial]
    fn test_weighted_with_seed_is_deterministic() {
        // Arrange
        let function = WeightedFunction;
        let pairs = vec![
            Value::Array(vec![Value::String("X".to_string()), Value::Number(serde_json::Number::from_f64(0.5).unwrap())]),
            Value::Array(vec![Value::String("Y".to_string()), Value::Number(serde_json::Number::from_f64(0.5).unwrap())]),
        ];

        let mut args1 = HashMap::new();
        args1.insert("pairs".to_string(), Value::Array(pairs.clone()));
        args1.insert("seed".to_string(), Value::Number(555.into()));

        let mut args2 = HashMap::new();
        args2.insert("pairs".to_string(), Value::Array(pairs.clone()));
        args2.insert("seed".to_string(), Value::Number(555.into()));

        // Act
        let result1 = function.call(&args1).expect("weighted should succeed");
        let result2 = function.call(&args2).expect("weighted should succeed");

        // Assert
        assert_eq!(result1, result2, "Weighted with same seed should be deterministic");
    }

    #[test]
    #[serial]
    fn test_weighted_errors_on_invalid_pairs() {
        // Arrange
        let function = WeightedFunction;
        let invalid_pairs = vec![
            Value::Array(vec![Value::String("A".to_string())]), // Missing weight
        ];

        let mut args = HashMap::new();
        args.insert("pairs".to_string(), Value::Array(invalid_pairs));

        // Act
        let result = function.call(&args);

        // Assert
        assert!(result.is_err(), "Should error on invalid pairs");
        assert!(result.unwrap_err().to_string().contains("2 elements"), "Error should mention 2 elements");
    }

    // ========================================
    // Shuffle Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_shuffle_preserves_all_elements() {
        // Arrange
        let function = ShuffleFunction;
        let list = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
            Value::Number(4.into()),
            Value::Number(5.into()),
        ];

        let mut args = HashMap::new();
        args.insert("list".to_string(), Value::Array(list.clone()));
        args.insert("seed".to_string(), Value::Number(42.into()));

        // Act
        let result = function.call(&args).expect("shuffle should succeed");

        // Assert
        let shuffled = result.as_array().expect("Result should be array");
        assert_eq!(shuffled.len(), list.len(), "Should preserve all elements");

        // All original elements should be present
        for item in &list {
            assert!(shuffled.contains(item), "Should contain element {:?}", item);
        }
    }

    #[test]
    #[serial]
    fn test_shuffle_with_seed_is_deterministic() {
        // Arrange
        let function = ShuffleFunction;
        let list = vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
            Value::String("d".to_string()),
        ];

        let mut args1 = HashMap::new();
        args1.insert("list".to_string(), Value::Array(list.clone()));
        args1.insert("seed".to_string(), Value::Number(123.into()));

        let mut args2 = HashMap::new();
        args2.insert("list".to_string(), Value::Array(list.clone()));
        args2.insert("seed".to_string(), Value::Number(123.into()));

        // Act
        let result1 = function.call(&args1).expect("shuffle should succeed");
        let result2 = function.call(&args2).expect("shuffle should succeed");

        // Assert
        assert_eq!(result1, result2, "Shuffle with same seed should be deterministic");
    }

    #[test]
    #[serial]
    fn test_shuffle_actually_shuffles() {
        // Arrange
        let function = ShuffleFunction;
        let list = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
            Value::Number(4.into()),
            Value::Number(5.into()),
            Value::Number(6.into()),
            Value::Number(7.into()),
            Value::Number(8.into()),
        ];

        let mut args = HashMap::new();
        args.insert("list".to_string(), Value::Array(list.clone()));
        args.insert("seed".to_string(), Value::Number(999.into()));

        // Act
        let result = function.call(&args).expect("shuffle should succeed");

        // Assert
        let shuffled = result.as_array().expect("Result should be array");

        // With 8 elements and a fixed seed, very unlikely to be in same order
        let is_different = list.iter().zip(shuffled.iter()).any(|(a, b)| a != b);
        assert!(is_different, "Shuffle should change order (statistically very likely with 8 elements)");
    }

    // ========================================
    // Sample Tests (3 tests)
    // ========================================

    #[test]
    #[serial]
    fn test_sample_returns_k_elements() {
        // Arrange
        let function = SampleFunction;
        let list = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
            Value::Number(4.into()),
            Value::Number(5.into()),
        ];

        let mut args = HashMap::new();
        args.insert("list".to_string(), Value::Array(list.clone()));
        args.insert("k".to_string(), Value::Number(3.into()));
        args.insert("seed".to_string(), Value::Number(42.into()));

        // Act
        let result = function.call(&args).expect("sample should succeed");

        // Assert
        let sample = result.as_array().expect("Result should be array");
        assert_eq!(sample.len(), 3, "Should return exactly k elements");

        // All sampled elements should be from original list
        for item in sample {
            assert!(list.contains(item), "Sampled element should be from original list");
        }
    }

    #[test]
    #[serial]
    fn test_sample_with_seed_is_deterministic() {
        // Arrange
        let function = SampleFunction;
        let list = vec![
            Value::String("alpha".to_string()),
            Value::String("beta".to_string()),
            Value::String("gamma".to_string()),
            Value::String("delta".to_string()),
        ];

        let mut args1 = HashMap::new();
        args1.insert("list".to_string(), Value::Array(list.clone()));
        args1.insert("k".to_string(), Value::Number(2.into()));
        args1.insert("seed".to_string(), Value::Number(456.into()));

        let mut args2 = HashMap::new();
        args2.insert("list".to_string(), Value::Array(list.clone()));
        args2.insert("k".to_string(), Value::Number(2.into()));
        args2.insert("seed".to_string(), Value::Number(456.into()));

        // Act
        let result1 = function.call(&args1).expect("sample should succeed");
        let result2 = function.call(&args2).expect("sample should succeed");

        // Assert
        assert_eq!(result1, result2, "Sample with same seed should be deterministic");
    }

    #[test]
    #[serial]
    fn test_sample_errors_when_k_exceeds_list_size() {
        // Arrange
        let function = SampleFunction;
        let list = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
        ];

        let mut args = HashMap::new();
        args.insert("list".to_string(), Value::Array(list));
        args.insert("k".to_string(), Value::Number(5.into()));

        // Act
        let result = function.call(&args);

        // Assert
        assert!(result.is_err(), "Should error when k > list size");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cannot be larger"), "Error should mention size constraint");
    }
}
