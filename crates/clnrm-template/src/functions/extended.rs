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

    // String transforms (keep functions for backward compatibility)
    tera.register_function("slug", SlugFunction);
    tera.register_function("kebab", KebabFunction);
    tera.register_function("snake", SnakeFunction);

    // String transforms as filters (ggen-style filter syntax)
    register_string_filters(tera);

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

/// Register string transformation filters (ggen-style)
/// Usage: {{ 'Hello World' | kebab }} instead of {{ kebab(s='Hello World') }}
fn register_string_filters(tera: &mut Tera) {
    use inflector::cases::{camelcase, kebabcase, pascalcase, snakecase};

    // Core Inflector filters
    reg_str_filter(tera, "camel", camelcase::to_camel_case);
    reg_str_filter(tera, "pascal", pascalcase::to_pascal_case);
    reg_str_filter(tera, "snake", snakecase::to_snake_case);
    reg_str_filter(tera, "kebab", kebabcase::to_kebab_case);

    // Slug filter (kebab-case with alphanumeric filtering)
    reg_str_filter(tera, "slug", |s: &str| {
        let kebab = kebabcase::to_kebab_case(s);
        kebab
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    });

    // Additional useful filters
    reg_str_filter(tera, "upper", |s: &str| s.to_uppercase());
    reg_str_filter(tera, "lower", |s: &str| s.to_lowercase());
}

/// Helper to register string transformation filters
/// Pattern from ggen: https://github.com/seanchatmangpt/ggen
fn reg_str_filter<F>(tera: &mut Tera, name: &str, f: F)
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    tera.register_filter(
        name,
        move |v: &Value, _args: &HashMap<String, Value>| -> tera::Result<Value> {
            let input_str = match v.as_str() {
                Some(s) => s,
                None => &v.to_string(),
            };
            Ok(Value::String(f(input_str)))
        },
    );
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

        let mut counters = self
            .counters
            .lock()
            .map_err(|e| tera::Error::msg(format!("Failed to lock sequence counter: {}", e)))?;
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
                tera::Error::msg(format!(
                    "Invalid base32 index {} during ULID timestamp encoding",
                    idx
                ))
            })?;
            ulid.insert(0, ch);
            ts /= 32;
        }

        // Random part (80 bits, 16 base32 chars)
        for _ in 0..16 {
            let idx = rng.gen_range(0..32);
            let ch = base32.chars().nth(idx).ok_or_else(|| {
                tera::Error::msg(format!(
                    "Invalid base32 index {} during ULID random part generation",
                    idx
                ))
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
        Ok(values
            .last()
            .ok_or_else(|| {
                tera::Error::msg("weighted() internal error: empty values after weight calculation")
            })?
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
        use inflector::cases::kebabcase::to_kebab_case;

        let s = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("slug() requires 's' string parameter"))?;

        // Use kebab-case but remove non-alphanumeric (except hyphens)
        let kebab = to_kebab_case(s);
        let slug = kebab
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
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
        use inflector::cases::kebabcase::to_kebab_case;

        let s = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("kebab() requires 's' string parameter"))?;

        Ok(Value::String(to_kebab_case(s)))
    }
}

/// snake(s) - Convert to snake_case
struct SnakeFunction;
impl Function for SnakeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use inflector::cases::snakecase::to_snake_case;

        let s = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("snake() requires 's' string parameter"))?;

        Ok(Value::String(to_snake_case(s)))
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
