#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-test.rs");
    t.pass("tests/02-async.rs");
}
