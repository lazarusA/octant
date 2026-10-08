//! `catch_panic`: a panicking task becomes an error.

use super::executor::catch_panic;

#[test]
fn a_panic_becomes_an_error_with_its_message() {
    let result: Result<(), String> = catch_panic(|| panic!("photometric interpretation not found"));
    let err = result.expect_err("the panic is caught");
    assert!(
        err.contains("photometric interpretation not found"),
        "{err}"
    );
}

#[test]
fn results_pass_through() {
    assert_eq!(catch_panic(|| Ok::<_, String>(7)), Ok(7));
    assert_eq!(
        catch_panic(|| Err::<(), _>("no".to_string())),
        Err("no".to_string())
    );
}
