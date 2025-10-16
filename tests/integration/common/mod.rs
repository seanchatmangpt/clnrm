//! Common module for integration tests
//!
//! Re-exports all test utilities, helpers, factories, and fixtures.

pub mod helpers {
    include!("../helpers/mod.rs");
}

pub mod factories {
    include!("../factories/mod.rs");
}

pub mod fixtures {
    include!("../fixtures/mod.rs");
}

pub mod assertions {
    include!("../assertions/mod.rs");
}
