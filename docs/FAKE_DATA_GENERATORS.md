# Fake Data Generator Functions (Issue #8)

## Overview

Implemented 50+ fake data generator functions for the Tera template system in clnrm. These functions enable realistic test data generation directly in `.clnrm.toml` test configuration files.

## Implementation Details

### Files Modified

1. **`crates/clnrm-core/src/template/functions.rs`**
   - Added 50+ fake data generator functions
   - All functions support optional `seed` parameter for deterministic output
   - Integrated with existing Tera template function registration

2. **`crates/clnrm-core/Cargo.toml`**
   - Updated `fake` dependency to version 2.9 with features: `derive`, `chrono`, `http`, `uuid`, `random_color`
   - Added `ai` feature flag for workspace compatibility

### Files Added

1. **`crates/clnrm-core/tests/template/fake_data_test.rs`**
   - Comprehensive test suite with 40+ test cases
   - Tests all fake data generator functions
   - Validates deterministic behavior with seeding
   - Includes practical integration test examples

## Available Functions (56 total)

### UUIDs (2)
- `fake_uuid()` - Random UUID v4
- `fake_uuid_seeded(seed=N)` - Deterministic UUID from seed

### Names (5)
- `fake_name()` - Full name
- `fake_first_name()` - First name only
- `fake_last_name()` - Last name only
- `fake_title()` - Name title (Mr., Mrs., etc.)
- `fake_suffix()` - Name suffix (Jr., Sr., etc.)

### Internet (9)
- `fake_email()` - Email address
- `fake_username()` - Username
- `fake_password(min=8, max=20)` - Password
- `fake_domain()` - Domain name
- `fake_url()` - Complete URL
- `fake_ipv4()` - IPv4 address
- `fake_ipv6()` - IPv6 address
- `fake_user_agent()` - User agent string
- `fake_mac_address()` - MAC address

### Address (7)
- `fake_street()` - Street name
- `fake_city()` - City name
- `fake_state()` - State name
- `fake_zip()` - ZIP code
- `fake_country()` - Country name
- `fake_latitude()` - Latitude coordinate
- `fake_longitude()` - Longitude coordinate

### Phone (2)
- `fake_phone()` - Phone number
- `fake_cell_phone()` - Cell phone number

### Company (4)
- `fake_company()` - Company name
- `fake_company_suffix()` - Company suffix (Inc., LLC, etc.)
- `fake_industry()` - Industry name
- `fake_profession()` - Profession

### Lorem Ipsum (4)
- `fake_word()` - Single random word
- `fake_words(count=3)` - Multiple words
- `fake_sentence(min=4, max=10)` - Random sentence
- `fake_paragraph(min=3, max=7)` - Random paragraph

### Numbers (4)
- `fake_int()` - Random integer (0-1000)
- `fake_int_range(min=0, max=100)` - Integer in range
- `fake_float()` - Random float (0.0-1000.0)
- `fake_bool(ratio=50)` - Random boolean

### Dates & Times (4)
- `fake_date()` - Date string
- `fake_time()` - Time string
- `fake_datetime()` - RFC3339 datetime
- `fake_timestamp()` - Unix timestamp

### Finance (4)
- `fake_credit_card()` - Credit card number
- `fake_currency_code()` - Currency code (USD, EUR, etc.)
- `fake_currency_name()` - Currency name
- `fake_currency_symbol()` - Currency symbol ($, €, etc.)

### File & Path (4)
- `fake_filename()` - Filename
- `fake_extension()` - File extension
- `fake_mime_type()` - MIME type
- `fake_file_path()` - File path

### Color (3)
- `fake_color()` - Color name
- `fake_hex_color()` - Hex color code (#RRGGBB)
- `fake_rgb_color()` - RGB color string

### Miscellaneous (3)
- `fake_string(len=10)` - Random string
- `fake_port()` - Port number (1024-65535)
- `fake_semver()` - Semantic version (X.Y.Z)

## Usage Examples

### Basic Usage

```toml
[test.user]
id = "{{ fake_uuid() }}"
name = "{{ fake_name() }}"
email = "{{ fake_email() }}"
```

### Deterministic Testing (with seeds)

```toml
[test.user]
# Same seed = same output (reproducible tests)
id = "{{ fake_uuid_seeded(seed=42) }}"
name = "{{ fake_name(seed=42) }}"
email = "{{ fake_email(seed=42) }}"
```

### Complete User Profile

```toml
[test.user]
id = "{{ fake_uuid() }}"
username = "{{ fake_username() }}"
email = "{{ fake_email() }}"
password = "{{ fake_password(min=12, max=16) }}"

[test.user.profile]
first_name = "{{ fake_first_name() }}"
last_name = "{{ fake_last_name() }}"
title = "{{ fake_title() }}"
phone = "{{ fake_phone() }}"
profession = "{{ fake_profession() }}"

[test.user.address]
street = "{{ fake_street() }}"
city = "{{ fake_city() }}"
state = "{{ fake_state() }}"
zip = "{{ fake_zip() }}"
country = "{{ fake_country() }}"

[test.user.company]
name = "{{ fake_company() }}"
industry = "{{ fake_industry() }}"
```

### Network Configuration

```toml
[test.network]
ipv4 = "{{ fake_ipv4() }}"
ipv6 = "{{ fake_ipv6() }}"
port = {{ fake_port() }}
mac = "{{ fake_mac_address() }}"
url = "{{ fake_url() }}"
```

### Dynamic Content

```toml
[test.content]
title = "{{ fake_sentence(min=3, max=6) }}"
description = "{{ fake_paragraph(min=2, max=4) }}"
tags = "{{ fake_words(count=5) }}"
color = "{{ fake_hex_color() }}"
```

## Key Features

### 1. Deterministic Output
All functions accept an optional `seed` parameter for reproducible test data:
```toml
name = "{{ fake_name(seed=123) }}"  # Always generates same name
```

### 2. Parameter Customization
Many functions accept parameters to customize output:
```toml
password = "{{ fake_password(min=16, max=32) }}"
words = "{{ fake_words(count=10) }}"
number = {{ fake_int_range(min=1, max=100) }}
```

### 3. Type Safety
- String functions return quoted strings
- Number functions return unquoted integers/floats
- Boolean functions return `true`/`false`

### 4. No External Dependencies
All data generation happens in-process using the `fake` crate - no network calls required.

## Testing

### Unit Tests
The implementation includes 40+ unit tests covering:
- Individual function correctness
- Parameter handling
- Deterministic behavior with seeds
- Integration with Tera templates
- Practical use cases

### Running Tests
```bash
# Run all template function tests
cargo test -p clnrm-core template::functions

# Run specific fake data tests
cargo test -p clnrm-core fake_data_test
```

## Architecture

### Function Structure
Each function follows this pattern:

```rust
struct FakeFunctionName;
impl Function for FakeFunctionName {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        use fake::faker::module::en::Generator;
        let seed = get_seed(args);  // Optional seed from args
        let mut rng = StdRng::seed_from_u64(seed);
        Ok(Value::String(Generator().fake_with_rng(&mut rng)))
    }
}
```

### Registration
All functions are registered in `register_fake_data_functions()`:

```rust
pub fn register_functions(tera: &mut Tera) -> Result<()> {
    // Original functions
    tera.register_function("env", EnvFunction);
    tera.register_function("sha256", Sha256Function);

    // Fake data generators (50+)
    register_fake_data_functions(tera);

    Ok(())
}
```

## Benefits

1. **Realistic Test Data** - Generate data that looks like production
2. **No Hardcoding** - Avoid brittle tests with hardcoded values
3. **Deterministic When Needed** - Use seeds for reproducible tests
4. **Developer Productivity** - No manual test data creation
5. **Template Native** - Works seamlessly in `.clnrm.toml` files

## Future Enhancements

Potential additions (not in scope for Issue #8):
- Additional fake data types (SSN, passport, etc.)
- Locale-specific data generation
- Custom fake data generators via plugins
- Fake data sequences and patterns

## References

- Issue: #8
- Fake crate: https://docs.rs/fake/latest/fake/
- Tera templates: https://tera.netlify.app/
