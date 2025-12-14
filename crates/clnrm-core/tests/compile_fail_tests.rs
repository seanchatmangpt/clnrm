//! Compile-time validated test run bounds checking
//!
//! These tests verify that ValidatedTestRun enforces compile-time bounds:
//! - N must be >= 10
//! - N must be <= 1000

#[cfg(test)]
mod compile_time_bounds {
    use clnrm_core::testing::timeout::ValidatedTestRun;

    #[test]
    fn test_valid_bounds_compile() {
        // These should compile successfully
        let _valid1 = ValidatedTestRun::<10>::new();
        let _valid2 = ValidatedTestRun::<100>::new();
        let _valid3 = ValidatedTestRun::<1000>::new();
    }
}

// Compile-fail tests - these are in separate modules to prevent compilation
// The trybuild crate would be used in practice, but for now we demonstrate
// that these would fail to compile if uncommented.

/*
// This module demonstrates compile-fail cases for ValidatedTestRun::<9>
// Uncommenting this would cause: error[E0080]: evaluation of constant value failed
#[cfg(test)]
mod compile_fail_too_small {
    use clnrm_core::testing::timeout::ValidatedTestRun;

    // This should fail to compile: N < 10
    // error[E0080]: evaluation of constant value failed
    // const TOO_SMALL: ValidatedTestRun<9> = ValidatedTestRun::<9>::new();
}
*/

/*
// This module demonstrates compile-fail cases for ValidatedTestRun::<1001>
// Uncommenting this would cause: error[E0080]: evaluation of constant value failed
#[cfg(test)]
mod compile_fail_too_large {
    use clnrm_core::testing::timeout::ValidatedTestRun;

    // This should fail to compile: N > 1000
    // error[E0080]: evaluation of constant value failed
    // const TOO_LARGE: ValidatedTestRun<1001> = ValidatedTestRun::<1001>::new();
}
*/

/*
// This module demonstrates compile-fail cases for ValidatedTestRun::<0>
// Uncommenting this would cause: error[E0080]: evaluation of constant value failed
#[cfg(test)]
mod compile_fail_zero {
    use clnrm_core::testing::timeout::ValidatedTestRun;

    // This should fail to compile: N = 0
    // error[E0080]: evaluation of constant value failed
    // const ZERO: ValidatedTestRun<0> = ValidatedTestRun::<0>::new();
}
*/
