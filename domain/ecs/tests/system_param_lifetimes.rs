#[test]
fn command_lifetime_cannot_escape_system_invocation() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/system_param_command_escape.rs");
}
