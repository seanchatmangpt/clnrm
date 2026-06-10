use clnrm_core::backend::oci::ConfigParser;
use clnrm_core::config::parse_toml_config;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const ITERATIONS: usize = 10_000;

#[test]
fn fuzz_toml_parser() {
    let mut rng = StdRng::seed_from_u64(42);

    for _ in 0..ITERATIONS {
        // Generate random length between 0 and 1024
        let len = rng.gen_range(0..1024);
        let mut bytes = vec![0u8; len];
        rng.fill(bytes.as_mut_slice());

        // Convert to a valid UTF-8 string since the parser expects &str
        let string_data = String::from_utf8_lossy(&bytes);

        // Feed to the parser and ensure it returns an error gracefully without panicking
        let result = parse_toml_config(&string_data);

        // The random garbage is extremely unlikely to be a valid TOML configuration
        // In the extremely rare case it is valid, we don't assert it's an error,
        // but we just ensure it didn't panic.
        if result.is_ok() {
            // Unlikely, but not a failure if it's somehow valid TOML and TestConfig
        }
    }
}

#[test]
fn fuzz_oci_manifest_parser() {
    let mut rng = StdRng::seed_from_u64(43);

    for _ in 0..ITERATIONS {
        // Generate random length between 0 and 1024
        let len = rng.gen_range(0..1024);
        let mut bytes = vec![0u8; len];
        rng.fill(bytes.as_mut_slice());

        // Feed directly to the parser and ensure it returns an error gracefully without panicking
        let result = ConfigParser::parse(&bytes);

        // Should return an error gracefully
        if result.is_ok() {
            // Unlikely, but not a failure if it's somehow valid JSON and OciImageConfig
        }
    }
}
