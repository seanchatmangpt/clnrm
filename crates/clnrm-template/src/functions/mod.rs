//! Custom Tera functions for clnrm templates
//!
//! Provides built-in functions for template rendering:
//! - `env(name)` - Get environment variable
//! - `now_rfc3339()` - Current timestamp (respects freeze_clock)
//! - `sha256(s)` - SHA-256 hex digest
//! - `toml_encode(value)` - Encode as TOML literal
//! - `fake_name()` - Generate fake names for testing (test-only)
//! - `fake_email()` - Generate fake emails for testing (test-only)
//! - 50+ fake data generators for testing
//! - Extended functions: UUIDs, collections, OTEL helpers, etc.

pub mod extended;

use crate::error::Result;
use fake::Fake;
use rand::rngs::StdRng;
use rand::SeedableRng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Function, Tera, Value};

/// Register all custom functions with Tera
///
/// # Arguments
/// * `tera` - Tera template engine
/// * `determinism` - Optional determinism engine for reproducible rendering
pub fn register_functions(
    tera: &mut Tera,
    determinism: Option<Arc<dyn TimestampProvider + Send + Sync>>,
) -> Result<()> {
    // Original functions
    tera.register_function("env", EnvFunction);
    tera.register_function("now_rfc3339", NowRfc3339Function::new(determinism.clone()));
    tera.register_function("sha256", Sha256Function);
    tera.register_function("toml_encode", TomlEncodeFunction);

    // Fake data generators with determinism support
    register_fake_data_functions(tera, determinism.clone());

    // Extended functions (UUIDs, collections, OTEL, etc.)
    extended::register_extended_functions(tera);

    Ok(())
}

/// Trait for timestamp providers (for determinism support)
pub trait TimestampProvider {
    fn get_timestamp_rfc3339(&self) -> String;
}

/// Register all fake data generator functions
fn register_fake_data_functions(
    tera: &mut Tera,
    _determinism: Option<Arc<dyn TimestampProvider + Send + Sync>>,
) {
    // UUIDs
    tera.register_function("fake_uuid", FakeUuidFunction);
    tera.register_function("fake_uuid_seeded", FakeUuidSeededFunction);

    // Names
    tera.register_function("fake_name", FakeNameFunction);
    tera.register_function("fake_first_name", FakeFirstNameFunction);
    tera.register_function("fake_last_name", FakeLastNameFunction);
    tera.register_function("fake_title", FakeTitleFunction);
    tera.register_function("fake_suffix", FakeSuffixFunction);

    // Internet
    tera.register_function("fake_email", FakeEmailFunction);
    tera.register_function("fake_username", FakeUsernameFunction);
    tera.register_function("fake_password", FakePasswordFunction);
    tera.register_function("fake_domain", FakeDomainFunction);
    tera.register_function("fake_url", FakeUrlFunction);
    tera.register_function("fake_ipv4", FakeIpv4Function);
    tera.register_function("fake_ipv6", FakeIpv6Function);
    tera.register_function("fake_user_agent", FakeUserAgentFunction);
    tera.register_function("fake_mac_address", FakeMacAddressFunction);

    // Address
    tera.register_function("fake_street", FakeStreetFunction);
    tera.register_function("fake_city", FakeCityFunction);
    tera.register_function("fake_state", FakeStateFunction);
    tera.register_function("fake_zip", FakeZipFunction);
    tera.register_function("fake_country", FakeCountryFunction);
    tera.register_function("fake_latitude", FakeLatitudeFunction);
    tera.register_function("fake_longitude", FakeLongitudeFunction);

    // Phone
    tera.register_function("fake_phone", FakePhoneFunction);
    tera.register_function("fake_cell_phone", FakeCellPhoneFunction);

    // Company
    tera.register_function("fake_company", FakeCompanyFunction);
    tera.register_function("fake_company_suffix", FakeCompanySuffixFunction);
    tera.register_function("fake_industry", FakeIndustryFunction);
    tera.register_function("fake_profession", FakeProfessionFunction);

    // Lorem
    tera.register_function("fake_word", FakeWordFunction);
    tera.register_function("fake_words", FakeWordsFunction);
    tera.register_function("fake_sentence", FakeSentenceFunction);
    tera.register_function("fake_paragraph", FakeParagraphFunction);

    // Numbers
    tera.register_function("fake_int", FakeIntFunction);
    tera.register_function("fake_int_range", FakeIntRangeFunction);
    tera.register_function("fake_float", FakeFloatFunction);
    tera.register_function("fake_bool", FakeBoolFunction);

    // Dates & Times
    tera.register_function("fake_date", FakeDateFunction);
    tera.register_function("fake_time", FakeTimeFunction);
    tera.register_function("fake_datetime", FakeDateTimeFunction);
    tera.register_function("fake_timestamp", FakeTimestampFunction);

    // Finance
    tera.register_function("fake_credit_card", FakeCreditCardFunction);
    tera.register_function("fake_currency_code", FakeCurrencyCodeFunction);
    tera.register_function("fake_currency_name", FakeCurrencyNameFunction);
    tera.register_function("fake_currency_symbol", FakeCurrencySymbolFunction);

    // File & Path
    tera.register_function("fake_filename", FakeFilenameFunction);
    tera.register_function("fake_extension", FakeExtensionFunction);
    tera.register_function("fake_mime_type", FakeMimeTypeFunction);
    tera.register_function("fake_file_path", FakeFilePathFunction);

    // Color
    tera.register_function("fake_color", FakeColorFunction);
    tera.register_function("fake_hex_color", FakeHexColorFunction);
    tera.register_function("fake_rgb_color", FakeRgbColorFunction);

    // Misc
    tera.register_function("fake_string", FakeStringFunction);
    tera.register_function("fake_port", FakePortFunction);
    tera.register_function("fake_semver", FakeSemverFunction);
}

/// env(name) - Get environment variable
///
/// Usage: `{{ env(name="HOME") }}`
struct EnvFunction;

impl Function for EnvFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("env() requires 'name' parameter"))?;

        std::env::var(name)
            .map(Value::String)
            .map_err(|_| tera::Error::msg(format!("Environment variable '{}' not found", name)))
    }
}

/// now_rfc3339() - Current timestamp (respects freeze_clock)
///
/// Usage: `{{ now_rfc3339() }}`
///
/// Returns RFC3339 formatted timestamp. Can be frozen for deterministic tests.
struct NowRfc3339Function {
    provider: Option<Arc<dyn TimestampProvider + Send + Sync>>,
}

impl NowRfc3339Function {
    fn new(provider: Option<Arc<dyn TimestampProvider + Send + Sync>>) -> Self {
        Self { provider }
    }
}

impl Function for NowRfc3339Function {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(ref provider) = self.provider {
            Ok(Value::String(provider.get_timestamp_rfc3339()))
        } else {
            Ok(Value::String(chrono::Utc::now().to_rfc3339()))
        }
    }
}

/// sha256(s) - SHA-256 hex digest
///
/// Usage: `{{ sha256(s="hello") }}`
struct Sha256Function;

impl Function for Sha256Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let input = args
            .get("s")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("sha256() requires 's' parameter"))?;

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        Ok(Value::String(format!("{:x}", result)))
    }
}

/// toml_encode(value) - Encode as TOML literal
///
/// Usage: `{{ toml_encode(value=vars.myvar) }}`
struct TomlEncodeFunction;

impl Function for TomlEncodeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let value = args
            .get("value")
            .ok_or_else(|| tera::Error::msg("toml_encode() requires 'value' parameter"))?;

        // Convert JSON value to TOML string
        let toml_str = match value {
            Value::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
                        _ => v.to_string(),
                    })
                    .collect();
                format!("[{}]", items.join(","))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| match v {
                        Value::String(s) => {
                            format!("\"{}\"=\"{}\"", k, s.replace('\"', "\\\""))
                        }
                        _ => format!("\"{}\"={}", k, v),
                    })
                    .collect();
                format!("{{{}}}", items.join(","))
            }
            Value::Null => "null".to_string(),
        };

        Ok(Value::String(toml_str))
    }
}

// ========================================
// Fake Data Generator Functions (50+)
// ========================================

// Helper to get seed from args
fn get_seed(args: &HashMap<String, Value>) -> u64 {
    args.get("seed")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(rand::random)
}

// === UUIDs ===

/// fake_uuid() - Generate random UUID v4
struct FakeUuidFunction;
impl Function for FakeUuidFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        Ok(Value::String(uuid::Uuid::new_v4().to_string()))
    }
}

/// fake_uuid_seeded(seed=42) - Generate deterministic UUID from seed
struct FakeUuidSeededFunction;
impl Function for FakeUuidSeededFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        // Generate deterministic UUID from seed
        let uuid_bytes = format!("{:032x}", seed);
        Ok(Value::String(format!(
            "{}-{}-{}-{}-{}",
            &uuid_bytes[0..8],
            &uuid_bytes[8..12],
            &uuid_bytes[12..16],
            &uuid_bytes[16..20],
            &uuid_bytes[20..32]
        )))
    }
}

// === Names ===

/// fake_name() - Generate full name
struct FakeNameFunction;
impl Function for FakeNameFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::name::en::Name;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Name().fake_with_rng(&mut rng)))
    }
}

/// fake_first_name() - Generate first name
struct FakeFirstNameFunction;
impl Function for FakeFirstNameFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::name::en::FirstName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(FirstName().fake_with_rng(&mut rng)))
    }
}

/// fake_last_name() - Generate last name
struct FakeLastNameFunction;
impl Function for FakeLastNameFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::name::en::LastName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(LastName().fake_with_rng(&mut rng)))
    }
}

/// fake_title() - Generate name title (Mr., Mrs., etc.)
struct FakeTitleFunction;
impl Function for FakeTitleFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::name::en::Title;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Title().fake_with_rng(&mut rng)))
    }
}

/// fake_suffix() - Generate name suffix (Jr., Sr., etc.)
struct FakeSuffixFunction;
impl Function for FakeSuffixFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::name::en::Suffix;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Suffix().fake_with_rng(&mut rng)))
    }
}

// === Internet ===

/// fake_email() - Generate email address
struct FakeEmailFunction;
impl Function for FakeEmailFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::SafeEmail;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(SafeEmail().fake_with_rng(&mut rng)))
    }
}

/// fake_username() - Generate username
struct FakeUsernameFunction;
impl Function for FakeUsernameFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::Username;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Username().fake_with_rng(&mut rng)))
    }
}

/// fake_password(min=8, max=20) - Generate password
struct FakePasswordFunction;
impl Function for FakePasswordFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::Password;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let min = args.get("min").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
        let max = args.get("max").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
        Ok(Value::String(Password(min..max).fake_with_rng(&mut rng)))
    }
}

/// fake_domain() - Generate domain name
struct FakeDomainFunction;
impl Function for FakeDomainFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::DomainSuffix;
        use fake::faker::lorem::en::Word;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let word: String = Word().fake_with_rng(&mut rng);
        let suffix: String = DomainSuffix().fake_with_rng(&mut rng);
        Ok(Value::String(format!("{}.{}", word, suffix)))
    }
}

/// fake_url() - Generate URL
struct FakeUrlFunction;
impl Function for FakeUrlFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::DomainSuffix;
        use fake::faker::lorem::en::Word;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let word: String = Word().fake_with_rng(&mut rng);
        let suffix: String = DomainSuffix().fake_with_rng(&mut rng);
        Ok(Value::String(format!("https://{}.{}", word, suffix)))
    }
}

/// fake_ipv4() - Generate IPv4 address
struct FakeIpv4Function;
impl Function for FakeIpv4Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::IPv4;
        use fake::Fake;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let ip: std::net::Ipv4Addr = IPv4().fake_with_rng::<std::net::Ipv4Addr, _>(&mut rng);
        Ok(Value::String(ip.to_string()))
    }
}

/// fake_ipv6() - Generate IPv6 address
struct FakeIpv6Function;
impl Function for FakeIpv6Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::IPv6;
        use fake::Fake;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let ip: std::net::Ipv6Addr = IPv6().fake_with_rng::<std::net::Ipv6Addr, _>(&mut rng);
        Ok(Value::String(ip.to_string()))
    }
}

/// fake_user_agent() - Generate user agent string
struct FakeUserAgentFunction;
impl Function for FakeUserAgentFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::UserAgent;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(UserAgent().fake_with_rng(&mut rng)))
    }
}

/// fake_mac_address() - Generate MAC address
struct FakeMacAddressFunction;
impl Function for FakeMacAddressFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::internet::en::MACAddress;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(MACAddress().fake_with_rng(&mut rng)))
    }
}

// === Address ===

/// fake_street() - Generate street address
struct FakeStreetFunction;
impl Function for FakeStreetFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::StreetName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(StreetName().fake_with_rng(&mut rng)))
    }
}

/// fake_city() - Generate city name
struct FakeCityFunction;
impl Function for FakeCityFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::CityName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CityName().fake_with_rng(&mut rng)))
    }
}

/// fake_state() - Generate state name
struct FakeStateFunction;
impl Function for FakeStateFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::StateName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(StateName().fake_with_rng(&mut rng)))
    }
}

/// fake_zip() - Generate ZIP code
struct FakeZipFunction;
impl Function for FakeZipFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::ZipCode;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(ZipCode().fake_with_rng(&mut rng)))
    }
}

/// fake_country() - Generate country name
struct FakeCountryFunction;
impl Function for FakeCountryFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::CountryName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CountryName().fake_with_rng(&mut rng)))
    }
}

/// fake_latitude() - Generate latitude
struct FakeLatitudeFunction;
impl Function for FakeLatitudeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::Latitude;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Latitude().fake_with_rng(&mut rng)))
    }
}

/// fake_longitude() - Generate longitude
struct FakeLongitudeFunction;
impl Function for FakeLongitudeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::address::en::Longitude;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Longitude().fake_with_rng(&mut rng)))
    }
}

// === Phone ===

/// fake_phone() - Generate phone number
struct FakePhoneFunction;
impl Function for FakePhoneFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::phone_number::en::PhoneNumber;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(PhoneNumber().fake_with_rng(&mut rng)))
    }
}

/// fake_cell_phone() - Generate cell phone number
struct FakeCellPhoneFunction;
impl Function for FakeCellPhoneFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::phone_number::en::CellNumber;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CellNumber().fake_with_rng(&mut rng)))
    }
}

// === Company ===

/// fake_company() - Generate company name
struct FakeCompanyFunction;
impl Function for FakeCompanyFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::company::en::CompanyName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CompanyName().fake_with_rng(&mut rng)))
    }
}

/// fake_company_suffix() - Generate company suffix (Inc., LLC, etc.)
struct FakeCompanySuffixFunction;
impl Function for FakeCompanySuffixFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::company::en::CompanySuffix;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CompanySuffix().fake_with_rng(&mut rng)))
    }
}

/// fake_industry() - Generate industry name
struct FakeIndustryFunction;
impl Function for FakeIndustryFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::company::en::Industry;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Industry().fake_with_rng(&mut rng)))
    }
}

/// fake_profession() - Generate profession
struct FakeProfessionFunction;
impl Function for FakeProfessionFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::company::en::Profession;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Profession().fake_with_rng(&mut rng)))
    }
}

// === Lorem ===

/// fake_word() - Generate random word
struct FakeWordFunction;
impl Function for FakeWordFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::lorem::en::Word;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Word().fake_with_rng(&mut rng)))
    }
}

/// fake_words(count=3) - Generate multiple words
struct FakeWordsFunction;
impl Function for FakeWordsFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::lorem::en::Words;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
        let words: Vec<String> = Words(count..count + 1).fake_with_rng(&mut rng);
        Ok(Value::String(words.join(" ")))
    }
}

/// fake_sentence() - Generate sentence
struct FakeSentenceFunction;
impl Function for FakeSentenceFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::lorem::en::Sentence;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let min = args.get("min").and_then(|v| v.as_u64()).unwrap_or(4) as usize;
        let max = args.get("max").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        Ok(Value::String(Sentence(min..max).fake_with_rng(&mut rng)))
    }
}

/// fake_paragraph() - Generate paragraph
struct FakeParagraphFunction;
impl Function for FakeParagraphFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::lorem::en::Paragraph;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let min = args.get("min").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
        let max = args.get("max").and_then(|v| v.as_u64()).unwrap_or(7) as usize;
        Ok(Value::String(Paragraph(min..max).fake_with_rng(&mut rng)))
    }
}

// === Numbers ===

/// fake_int() - Generate random integer
struct FakeIntFunction;
impl Function for FakeIntFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let value: i32 = (0..1000).fake_with_rng(&mut rng);
        Ok(Value::Number(value.into()))
    }
}

/// fake_int_range(min=0, max=100) - Generate integer in range
struct FakeIntRangeFunction;
impl Function for FakeIntRangeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let min = args.get("min").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let max = args.get("max").and_then(|v| v.as_i64()).unwrap_or(100) as i32;
        let value: i32 = (min..max).fake_with_rng(&mut rng);
        Ok(Value::Number(value.into()))
    }
}

/// fake_float() - Generate random float
struct FakeFloatFunction;
impl Function for FakeFloatFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let value: f64 = (0.0..1000.0).fake_with_rng(&mut rng);
        Ok(Value::Number(
            serde_json::Number::from_f64(value).unwrap_or(serde_json::Number::from(0)),
        ))
    }
}

/// fake_bool() - Generate random boolean
struct FakeBoolFunction;
impl Function for FakeBoolFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::boolean::en::Boolean;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let ratio = args.get("ratio").and_then(|v| v.as_u64()).unwrap_or(50) as u8;
        Ok(Value::Bool(Boolean(ratio).fake_with_rng(&mut rng)))
    }
}

// === Dates & Times ===

/// fake_date() - Generate date string
struct FakeDateFunction;
impl Function for FakeDateFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::chrono::en::Date;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let date: chrono::NaiveDate = Date().fake_with_rng(&mut rng);
        Ok(Value::String(date.to_string()))
    }
}

/// fake_time() - Generate time string
struct FakeTimeFunction;
impl Function for FakeTimeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::chrono::en::Time;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let time: chrono::NaiveTime = Time().fake_with_rng(&mut rng);
        Ok(Value::String(time.to_string()))
    }
}

/// fake_datetime() - Generate datetime string
struct FakeDateTimeFunction;
impl Function for FakeDateTimeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::chrono::en::DateTime;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let dt: chrono::DateTime<chrono::Utc> = DateTime().fake_with_rng(&mut rng);
        Ok(Value::String(dt.to_rfc3339()))
    }
}

/// fake_timestamp() - Generate Unix timestamp
struct FakeTimestampFunction;
impl Function for FakeTimestampFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::chrono::en::DateTime;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let dt: chrono::DateTime<chrono::Utc> = DateTime().fake_with_rng(&mut rng);
        Ok(Value::Number(dt.timestamp().into()))
    }
}

// === Finance ===

/// fake_credit_card() - Generate credit card number
struct FakeCreditCardFunction;
impl Function for FakeCreditCardFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::creditcard::en::CreditCardNumber;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CreditCardNumber().fake_with_rng(&mut rng)))
    }
}

/// fake_currency_code() - Generate currency code (USD, EUR, etc.)
struct FakeCurrencyCodeFunction;
impl Function for FakeCurrencyCodeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::currency::en::CurrencyCode;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CurrencyCode().fake_with_rng(&mut rng)))
    }
}

/// fake_currency_name() - Generate currency name
struct FakeCurrencyNameFunction;
impl Function for FakeCurrencyNameFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::currency::en::CurrencyName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CurrencyName().fake_with_rng(&mut rng)))
    }
}

/// fake_currency_symbol() - Generate currency symbol ($, €, etc.)
struct FakeCurrencySymbolFunction;
impl Function for FakeCurrencySymbolFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::currency::en::CurrencySymbol;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(CurrencySymbol().fake_with_rng(&mut rng)))
    }
}

// === File & Path ===

/// fake_filename() - Generate filename
struct FakeFilenameFunction;
impl Function for FakeFilenameFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::filesystem::en::FileName;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(FileName().fake_with_rng(&mut rng)))
    }
}

/// fake_extension() - Generate file extension
struct FakeExtensionFunction;
impl Function for FakeExtensionFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::filesystem::en::FileExtension;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(FileExtension().fake_with_rng(&mut rng)))
    }
}

/// fake_mime_type() - Generate MIME type
struct FakeMimeTypeFunction;
impl Function for FakeMimeTypeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::filesystem::en::MimeType;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(MimeType().fake_with_rng(&mut rng)))
    }
}

/// fake_file_path() - Generate file path
struct FakeFilePathFunction;
impl Function for FakeFilePathFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::filesystem::en::FilePath;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(FilePath().fake_with_rng(&mut rng)))
    }
}

// === Color ===

/// fake_color() - Generate color name
struct FakeColorFunction;
impl Function for FakeColorFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::color::en::Color;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Color().fake_with_rng(&mut rng)))
    }
}

/// fake_hex_color() - Generate hex color code
struct FakeHexColorFunction;
impl Function for FakeHexColorFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::color::en::HexColor;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(HexColor().fake_with_rng(&mut rng)))
    }
}

/// fake_rgb_color() - Generate RGB color
struct FakeRgbColorFunction;
impl Function for FakeRgbColorFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::color::en::RgbColor;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(RgbColor().fake_with_rng(&mut rng)))
    }
}

// === Misc ===

/// fake_string(len=10) - Generate random string
struct FakeStringFunction;
impl Function for FakeStringFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use rand::Rng;
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let len = args.get("len").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        let s: String = (0..len)
            .map(|_| rng.gen_range(b'a'..=b'z') as char)
            .collect();
        Ok(Value::String(s))
    }
}

/// fake_port() - Generate port number
struct FakePortFunction;
impl Function for FakePortFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let port: u16 = (1024..65535).fake_with_rng(&mut rng);
        Ok(Value::Number(port.into()))
    }
}

/// fake_semver() - Generate semantic version
struct FakeSemverFunction;
impl Function for FakeSemverFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let seed = get_seed(args);
        let mut rng = StdRng::seed_from_u64(seed);
        let major: u8 = (0..10).fake_with_rng(&mut rng);
        let minor: u8 = (0..20).fake_with_rng(&mut rng);
        let patch: u8 = (0..100).fake_with_rng(&mut rng);
        Ok(Value::String(format!("{}.{}.{}", major, minor, patch)))
    }
}
