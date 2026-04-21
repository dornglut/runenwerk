#[test]
fn id_macro_rejects_misuse_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
