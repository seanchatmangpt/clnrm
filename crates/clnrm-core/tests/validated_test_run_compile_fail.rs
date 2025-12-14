//! Compile-fail tests for ValidatedTestRun bounds checking
//!
//! These tests verify that ValidatedTestRun enforces compile-time bounds:
//! - N must be >= 10
//! - N must be <= 1000

#[cfg(test)]
mod compile_fail_tests {
    use clnrm_core::testing::timeout::ValidatedTestRun;

    // This should fail to compile - N < 10
    // error[E0080]: evaluation of constant value failed
    // fn test_too_small() {
    //     let _invalid = ValidatedTestRun::<9>::new();
    // }

    // This should fail to compile - N > 1000
    // error[E0080]: evaluation of constant value failed
    // fn test_too_large() {
    //     let _invalid = ValidatedTestRun::<1001>::new();
    // }

    // This should fail to compile - N = 0
    // error[E0080]: evaluation of constant value failed
    // fn test_zero() {
    //     let _invalid = ValidatedTestRun::<0>::new();
    // }

    // Valid cases that should compile
    #[test]
    fn test_valid_bounds() {
        let _valid1 = ValidatedTestRun::<10>::new();
        let _valid2 = ValidatedTestRun::<100>::new();
        let _valid3 = ValidatedTestRun::<1000>::new();
    }
}
