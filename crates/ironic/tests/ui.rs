//! Compile-time contracts for the facade's procedural macros.

#[test]
fn macro_contracts() {
    let tests = trybuild::TestCases::new();
    tests.pass("crates/ironic/tests/ui/pass/*.rs");
    tests.compile_fail("crates/ironic/tests/ui/fail/*.rs");
}
