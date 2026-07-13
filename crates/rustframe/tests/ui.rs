//! Compile-time contracts for the facade's procedural macros.

#[test]
fn macro_contracts() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass/*.rs");
    tests.compile_fail("tests/ui/fail/*.rs");
}
